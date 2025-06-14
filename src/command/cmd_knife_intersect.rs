// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model, OwnedModel};
use crate::{HallrError, command::Options, ffi, ffi::FFIVector3};
use hronn::prelude::ConvertTo;
use itertools::Itertools;
use linestring::linestring_2d::indexed_intersection::IntersectionTester;
use vector_traits::{
    approx::{AbsDiffEq, UlpsEq},
    num_traits::AsPrimitive,
    prelude::{Aabb3, GenericVector2, GenericVector3, HasXY, Plane},
};

#[cfg(test)]
mod tests;

/// detect self intersections and cut those lines at the intersection
fn knife_intersect<T>(input_model: &Model<'_>) -> Result<OwnedModel, HallrError>
where
    T: GenericVector3,
    FFIVector3: ConvertTo<T>,
    f32: AsPrimitive<T::Scalar>,
    T: ConvertTo<FFIVector3>,
{
    let mut aabb = <T as GenericVector3>::Aabb::default();
    for v in input_model.vertices.iter() {
        aabb.add_point(v.to())
    }

    let plane = aabb.get_plane_relaxed(f32::default_epsilon().as_(), f32::default_max_ulps()).ok_or_else(|| {
        let aabbe_d: T = aabb.max() - aabb.min();
        let aabbe_c: T =aabb.center();
        HallrError::InputNotPLane(format!(
            "Input data not in one plane and/or plane not intersecting origin: Δ({},{},{}) C({},{},{})",
            aabbe_d.x(), aabbe_d.y(), aabbe_d.z(), aabbe_c.x(), aabbe_c.y(), aabbe_c.z()
        ))
    })?;
    if plane != Plane::XY {
        return Err(HallrError::InvalidInputData(format!(
            "At the moment the knife intersect operation only supports input data in the XY plane. {plane:?}",
        )));
    }
    println!("knife_intersect: data was in plane:{plane:?} aabb:{aabb:?}",);
    //println!("input Lines:{:?}", input_pb_model.vertices);

    let vertices_2d: Vec<T::Vector2> = input_model
        .vertices
        .iter()
        .map(|v| -> T::Vector2 {
            let v: T = v.to();
            let v: T::Vector2 = plane.point_to_2d::<T>(v);
            v
        })
        .collect();

    let input_edges: Vec<(usize, usize)> = input_model
        .indices
        .chunks(2)
        .map(|i| (i[0], i[1]))
        .collect();
    println!("Input edges : {:?}", input_edges.len());

    // this map contains a map from `edge_id` ->  `SmallVec<new intersecting vertices id>`
    let mut edge_split = ahash::AHashMap::<usize, smallvec::SmallVec<[usize; 1]>>::default();
    let new_vertices = {
        let (updated_vertices_list, intersection_iter) =
            IntersectionTester::<T::Vector2>::new(vertices_2d)
                .with_ignore_end_point_intersections(true)?
                .with_stop_at_first_intersection(false)?
                .with_edges(input_edges.iter())?
                .compute()?;
        if intersection_iter.len() == 0 {
            println!("No intersections detected!!");
        }
        for (splitting_vertex_index, affected_edges) in intersection_iter {
            let splitting_vertex = updated_vertices_list[splitting_vertex_index];
            /*println!(
                "Intersection detected @({},{}):idx:{} Involved edges:{:?}",
                splitting_vertex.x(),
                splitting_vertex.y(),
                splitting_vertex_index,
                affected_edges
            );*/
            for edge_index in affected_edges.iter() {
                if !splitting_vertex.is_finite() {
                    return Err(HallrError::InternalError(format!(
                        "The found intersection is not valid: x:{:?}, y:{:?}",
                        splitting_vertex.x(),
                        splitting_vertex.y()
                    )));
                }
                edge_split
                    .entry(*edge_index)
                    .or_insert_with(smallvec::SmallVec::<[usize; 1]>::new)
                    .push(splitting_vertex_index);
            }
        }
        updated_vertices_list
    };

    let estimated_edges = input_edges.len() * 2 + edge_split.len();

    let mut output_model = {
        let world_orientation = input_model.copy_world_orientation()?;

        // Create vertices with the appropriate transformation
        let vertices = if let Some(world_to_local) = input_model.get_world_to_local_transform()? {
            println!(
                "Rust: applying world-local transformation 1/{:?}",
                input_model.world_orientation
            );
            new_vertices
                .into_iter()
                .map(|v| world_to_local(plane.point_to_3d::<T>(v).to()))
                .collect()
        } else {
            println!("Rust: *not* applying world-local transformation");
            new_vertices
                .into_iter()
                .map(|v| plane.point_to_3d::<T>(v).to())
                .collect()
        };

        OwnedModel {
            world_orientation,
            vertices,
            indices: Vec::<usize>::with_capacity(estimated_edges),
        }
    };

    // insert the un-affected edges into the output
    for (edge_id, edge) in input_edges.iter().enumerate() {
        if !edge_split.contains_key(&(edge_id)) {
            output_model.indices.push(edge.0);
            output_model.indices.push(edge.1);
            //println!("added un-affected edge: v:{}-v:{}", edge.0, edge.1)
        }
    }

    // output_model now contains a copy of input_model except for the edges with an intersection
    // Add the intersecting edges, but split them first

    for (edge_id, mut split_points) in edge_split {
        let (i0, i1) = input_edges[edge_id];
        let v0: T::Vector2 = output_model.vertices[i0].to().to_2d();
        /*println!();
        println!(
            "processing edge:{} split_points:{:?} i0:{}, v0:{:?}, i1:{}, v1:{:?}",
            edge_id,
            split_points,
            i0,
            v0,
            i1,
            output_model.vertices[i1].to().to_2d()
        );*/
        if !split_points.is_empty() {
            split_points.push(i0);
            split_points.push(i1);
            //output_model.indices.push(i0);
            //println!("split_points:{:?}", split_points);
            let new_vec: Vec<(usize, T::Vector2)> = split_points
                .into_iter()
                .map(|i| (i, output_model.vertices[i].to().to_2d()))
                .collect();
            //println!("new_vec:{:?}", new_vec);
            //println!("pushed: {}", i0);
            new_vec
                .into_iter()
                .sorted_unstable_by(|a, b| {
                    PartialOrd::partial_cmp(&v0.distance_sq(a.1), &v0.distance_sq(b.1)).unwrap()
                })
                .tuple_windows::<(_, _)>()
                .for_each(|(a, b)| {
                    output_model.indices.push(a.0);
                    //println!("pushed: {}", a.0);
                    output_model.indices.push(b.0);
                    //println!("pushed: {}", b.0);
                })
        }
    }

    //println!("estimated_edges:{}", estimated_edges);
    Ok(output_model)
}

pub(crate) fn process_command<T>(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: GenericVector3,
    T::Scalar: UlpsEq,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    f32: AsPrimitive<T::Scalar>,
{
    if models.is_empty() {
        return Err(HallrError::InvalidInputData(
            "No models detected".to_string(),
        ));
    }
    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Edges)?;

    let input_model = &models[0];
    /*if !input_model.has_xy_transform_only() {
        return Err(HallrError::InvalidInputData(
            "The knife_intersect operation currently only supports world transformations in the XY plane"
                .to_string(),
        ));
    }*/
    println!(
        "knife_intersect receiving {} vertices, {} indices, {} edges",
        input_model.vertices.len(),
        input_model.indices.len(),
        input_model.indices.chunks(2).count()
    );

    let rv_model = knife_intersect(input_model)?;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Edges.to_string(),
    );
    println!(
        "knife_intersect returning {} vertices, {} indices, {} edges",
        rv_model.vertices.len(),
        rv_model.indices.len(),
        rv_model.indices.chunks(2).count()
    );
    Ok((
        rv_model.vertices,
        rv_model.indices,
        rv_model.world_orientation.to_vec(),
        return_config,
    ))
}
