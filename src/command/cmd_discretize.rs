// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use crate::{
    HallrError,
    command::{ConfigType, Model, Options, OwnedModel},
    ffi,
    ffi::FFIVector3,
    utils::VertexDeduplicator3D,
};
use itertools::Itertools;
use linestring::linestring_3d::LineString3;
use std::time;
use vector_traits::{
    glam,
    prelude::{Aabb3, GenericVector3},
};

/// Build the return model
pub(crate) fn build_output_model(
    descretization_length_factor: f32,
    model: &Model<'_>,
    verbose: bool,
) -> Result<OwnedModel, HallrError> {
    let mut vertices = Vec::with_capacity(model.vertices.len());
    let indices = model.indices.to_vec();
    let mut v_dedup = VertexDeduplicator3D::with_capacity(vertices.len());
    let mut aabb = <glam::Vec3 as GenericVector3>::Aabb::default();

    for vertex in model.vertices.iter() {
        if !vertex.is_finite() {
            Err(HallrError::InvalidInputData(format!(
                "Only finite coordinates are allowed ({},{},{})",
                vertex.x, vertex.y, vertex.z
            )))?
        } else {
            let point = glam::vec3(vertex.x, vertex.y, vertex.z);
            aabb.add_point(point);
            vertices.push(point);
        }
    }

    let now = time::Instant::now();
    let descretization_length = {
        let extent = aabb.extents().2;
        extent.x.max(extent.y).max(extent.z) * descretization_length_factor
    };

    let mut out_indices = Vec::<u32>::with_capacity(indices.len());

    let (shapes, visited) = linestring::prelude::divide_into_shapes(&indices);
    for index in visited.iter_unset_bits(..) {
        let _ = v_dedup.get_index_or_insert(vertices[index])?;
    }

    for shape in shapes {
        let line: Vec<glam::Vec3> = shape.into_iter().map(|i| vertices[i as usize]).collect();
        let mut iter = line
            .discretize(descretization_length)
            .tuple_windows::<(_, _)>()
            .peekable();
        if let Some((v0, v1)) = iter.next() {
            let mut i0 = v_dedup.get_index_or_insert(v0)?;
            out_indices.push(i0);
            let mut i1 = if iter.peek().is_some() {
                v_dedup.insert_and_get_index(v1)
            } else {
                v_dedup.get_index_or_insert(v1)?
            };
            out_indices.push(i1);

            while iter.peek().is_some() {
                i0 = i1;
                out_indices.push(i0);

                let (_, v1) = iter.next().unwrap();
                i1 = if iter.peek().is_some() {
                    v_dedup.insert_and_get_index(v1)
                } else {
                    v_dedup.get_index_or_insert(v1)?
                };
                out_indices.push(i1);
            }
        }
    }

    if verbose {
        println!(
            "Rust: Vertex return model packaging duration: {:?}",
            now.elapsed()
        );
    }
    if let Some(world_to_local) = model.get_world_to_local_transform()? {
        println!(
            "Rust: applying world-local transformation 1/{:?}",
            model.world_orientation
        );
        Ok(OwnedModel {
            world_orientation: OwnedModel::identity_matrix(),
            //name: pb_model_name,
            vertices: v_dedup
                .vertices
                .into_iter()
                .map(|v| world_to_local(FFIVector3::new(v.x, v.y, v.z)))
                .collect(),
            indices: out_indices,
        })
    } else {
        println!("Rust: *not* applying world-local transformation");
        Ok(OwnedModel {
            world_orientation: OwnedModel::identity_matrix(),
            //name: pb_model_name,
            vertices: v_dedup
                .vertices
                .into_iter()
                .map(|v| FFIVector3::new(v.x, v.y, v.z))
                .collect(),
            indices: out_indices,
        })
    }
}

/// Run the voronoi_mesh command
pub(crate) fn process_command(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    if models.is_empty() {
        return Err(HallrError::InvalidInputData(
            "This operation requires one input model".to_string(),
        ));
    }

    if models.len() > 1 {
        return Err(HallrError::InvalidInputData(
            "This operation only supports one model as input".to_string(),
        ));
    }
    if models[0].vertices.is_empty() {
        return Err(HallrError::InvalidInputData(
            "Input vertex list was empty".to_string(),
        ));
    }
    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Edges)?;

    let cmd_arg_discretize_length_multiplier =
        input_config.get_mandatory_parsed_option::<f32>("discretize_length", None)? / 100.0;

    // we already tested a_command.models.len()
    let input_model = &models[0];

    println!(
        "Rust: model.vertices:{:?}, cmd_arg_discretize_length_multiplier:{}",
        input_model.vertices.len(),
        cmd_arg_discretize_length_multiplier
    );
    let output_model = build_output_model(cmd_arg_discretize_length_multiplier, input_model, true)?;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Edges.to_string(),
    );

    if let Some(mv) = input_config.get_optional_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }
    println!(
        "cmd discretize returning {} vertices, {} indices",
        output_model.vertices.len(),
        output_model.indices.len()
    );
    Ok((
        output_model.vertices,
        output_model.indices,
        output_model.world_orientation.to_vec(),
        return_config,
    ))
}
