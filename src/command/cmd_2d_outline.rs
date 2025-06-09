// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, Model, Options, OwnedModel},
    ffi,
    prelude::FFIVector3,
};
use hronn::prelude::ConvertTo;
use itertools::Itertools;
use vector_traits::{
    approx::{AbsDiffEq, UlpsEq},
    prelude::{Aabb3, GenericVector3, HasXY, HasXYZ},
};

#[cfg(test)]
mod tests;

#[inline(always)]
/// make a key from v0 and v1, lowest index will always be first
fn make_edge_key(v0: u32, v1: u32) -> (u32, u32) {
    if v0 < v1 { (v0, v1) } else { (v1, v0) }
}

#[allow(clippy::type_complexity)]
/// remove internal edges from the input model
fn remove_internal_edges<T: GenericVector3>(
    model: &Model<'_>,
) -> Result<(Vec<(u32, u32)>, Vec<FFIVector3>), HallrError>
where
    FFIVector3: ConvertTo<T>,
{
    let mut all_edges = ahash::AHashSet::<(u32, u32)>::default();
    //let mut single_edges = ahash::AHashSet::<(usize, usize)>::default();
    let mut internal_edges = ahash::AHashSet::<(u32, u32)>::default();
    //println!("Input faces : {:?}", obj.faces);

    let mut aabb = <T as GenericVector3>::Aabb::default();
    for v in model.vertices.iter() {
        aabb.add_point(v.to())
    }
    let plane =
        aabb.get_plane_relaxed(T::Scalar::default_epsilon(), T::Scalar::default_max_ulps()).ok_or_else(|| {
            let aabbe_d = aabb.max() - aabb.min();
            let aabbe_c =aabb.center();
            HallrError::InputNotPLane(format!(
                "Input data not in one plane and/or plane not intersecting origin: Δ({},{},{}) C({},{},{})",
                aabbe_d.x(), aabbe_d.y(), aabbe_d.z(),aabbe_c.x(), aabbe_c.y(), aabbe_c.z()
            ))
        })?;

    println!("2d_outline: data was in plane:{plane:?} aabb:{aabb:?}");

    for face in model.indices.chunks(3) {
        for (v0, v1) in face.iter().chain(face.first()).tuple_windows::<(_, _)>() {
            let v0 = *v0;
            let v1 = *v1;
            if v0 == v1 {
                return Err(HallrError::InvalidInputData(
                    "A face contained the same vertex at least twice".to_string(),
                ));
            }
            let key = make_edge_key(v0 as u32, v1 as u32);

            if all_edges.contains(&key) {
                let _ = internal_edges.insert(key);
            } else {
                let _ = all_edges.insert(key);
            }
        }
    }

    println!("Rust: Input vertices : {:?}", model.vertices.len());
    println!("Rust: Input internal edges: {:?}", internal_edges.len());
    println!("Rust: Input all edges: {:?}", all_edges.len());

    let kept_edges = all_edges
        .into_iter()
        .filter(|x| !internal_edges.contains(x))
        .collect();
    all_edges = kept_edges;

    /* for e in single_edges.into_iter() {
        let _ = all_edges.insert(e);
    }*/

    /*println!("All edges post: ");
    for (n, v) in all_edges.iter().enumerate() {
        println!("#{}, {:?}", n, v);
    }*/
    /*println!("Input all edges post filter: {:?}", all_edges.len());
    println!();
    */
    // all_edges should now contain the outline and none of the internal edges.
    // no need for internal_edges any more
    drop(internal_edges);
    // vector number translation table
    let mut vector_rename_map = ahash::AHashMap::<u32, u32>::default();
    let mut rv_vertices = Vec::<FFIVector3>::with_capacity(all_edges.len() * 6 / 5);
    let mut rv_lines = Vec::<(u32, u32)>::with_capacity(all_edges.len() * 6 / 5);

    // Iterate over each edge and store each used vertex (in no particular order)
    for (v0, v1) in all_edges {
        let v0 = if let Some(v0) = vector_rename_map.get(&v0) {
            *v0
        } else {
            let translated = (v0, rv_vertices.len() as u32);
            let _ = vector_rename_map.insert(translated.0, translated.1);
            let vtmp = &model.vertices[v0 as usize];
            rv_vertices.push(FFIVector3::new_3d(vtmp.x(), vtmp.y(), vtmp.z()));
            translated.1
        };
        let v1 = if let Some(v1) = vector_rename_map.get(&v1) {
            *v1
        } else {
            let translated = (v1, rv_vertices.len() as u32);
            let _ = vector_rename_map.insert(translated.0, translated.1);
            let vtmp = &model.vertices[v1 as usize];
            rv_vertices.push(FFIVector3::new_3d(vtmp.x(), vtmp.y(), vtmp.z()));
            translated.1
        };
        // v0 and v1 now contains the translated vertex indices.
        rv_lines.push((v0, v1));
    }
    if let Some(world_to_local) = model.get_world_to_local_transform()? {
        println!(
            "Rust: applying world-local transformation 1/{:?}",
            model.world_orientation
        );
        rv_vertices.iter_mut().for_each(|v| *v = world_to_local(*v));
    } else {
        println!("Rust: *not* applying world-local transformation");
    };

    Ok((rv_lines, rv_vertices))
}

/// Run the 2d_outline command
pub(crate) fn process_command<T>(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: GenericVector3,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
{
    if models.len() > 1 {
        return Err(HallrError::InvalidInputData(
            "This operation only supports one model as input".to_string(),
        ));
    }

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Triangulated)?;

    /*for model in models.iter() {
        //println!("model.name:{:?}, ", model.name);
        println!("model.vertices:{:?}, ", model.vertices.len());
        println!("model.indices:{:?}, ", model.indices.len());
        //println!(
        //    "model.world_orientation:{:?}, ",
        //    model.world_orientation.as_ref().map_or(0, |_| 16)
        //);
        println!();
    }*/
    if !models.is_empty() {
        let input_model = &models[0];
        let (rv_lines, rv_vector) = remove_internal_edges(input_model)?;

        let mut model = OwnedModel {
            //name: a_command.models[0].name.clone(),
            //world_orientation: input_model.world_orientation.clone(),
            world_orientation: input_model.copy_world_orientation()?,
            vertices: rv_vector,
            indices: Vec::<usize>::with_capacity(input_model.indices.len()),
        };
        for l in rv_lines.iter() {
            model.indices.push(l.0 as usize);
            model.indices.push(l.1 as usize);
        }
        let mut return_config = ConfigType::new();
        let _ = return_config.insert(
            ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
            ffi::MeshFormat::LineChunks.to_string(),
        );

        Ok((
            model.vertices,
            model.indices,
            model.world_orientation.to_vec(),
            return_config,
        ))
    } else {
        Err(HallrError::InvalidInputData(
            "Model did not contain any data".to_string(),
        ))
    }
}
