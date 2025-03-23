// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, prelude::FFIVector3, utils::IndexCompressor};
use baby_shark::{
    exports::nalgebra::Vector3,
    mesh::{corner_table::prelude::CornerTableF, traits::Mesh},
    remeshing::incremental::IncrementalRemesher,
};
use hronn::HronnError;
use std::time::Instant;

pub(crate) fn process_command(
    config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    if models.len() != 1 {
        Err(HronnError::InvalidParameter(
            "Incorrect number of models selected".to_string(),
        ))?
    }
    let model = &models[0];
    // todo: actually use the matrices
    let world_matrix = model.world_orientation.to_vec();

    // We simply have to clone the vertices here, because python still owns `model` and
    // from_vertices_and_indices() only accepts nalgebra::Vector3. We could avoid one copy
    // if a from_vertices_and_indices() variant could accept &[[f32;3]]
    // (&[FFIVector3] can easily be casted to &[[f32;3]]).
    let vertices_owned: Vec<Vector3<f32>> = model
        .vertices
        .iter()
        .map(|v| Vector3::new(v.x, v.y, v.z))
        .collect();

    let mut mesh = CornerTableF::from_vertices_and_indices(&vertices_owned, model.indices);
    let target_edge_length = config.get_mandatory_parsed_option("TARGET_EDGE_LENGTH", None)?;

    let remesher = IncrementalRemesher::new()
        .with_iterations_count(config.get_mandatory_parsed_option("ITERATIONS_COUNT", None)?)
        .with_split_edges(config.get_mandatory_parsed_option::<bool>("SPLIT_EDGES", Some(false))?)
        .with_collapse_edges(
            config.get_mandatory_parsed_option::<bool>("COLLAPSE_EDGES", Some(false))?,
        )
        .with_flip_edges(config.get_mandatory_parsed_option::<bool>("FLIP_EDGES", Some(false))?)
        .with_shift_vertices(
            config.get_mandatory_parsed_option::<bool>("SHIFT_VERTICES", Some(false))?,
        )
        .with_project_vertices(
            config.get_mandatory_parsed_option::<bool>("PROJECT_VERTICES", Some(false))?,
        );

    let start = Instant::now();
    remesher.remesh(&mut mesh, target_edge_length);
    println!("Rust: Time elapsed in remesh() was {:?}", start.elapsed());

    // it would be nice with a reverse of the `CornerTableF::from_vertices_and_indices()` method here.

    // Compress the ranges of the indices to a minimum
    let mut compressor = IndexCompressor::with_capacity(mesh.vertices().count());

    // Extract the triangles with remapped indices
    let mut ffi_indices = Vec::with_capacity(mesh.faces().count() * 3);

    for face_descriptor in mesh.faces() {
        let (v1, v2, v3) = mesh.face_vertices(&face_descriptor);

        // Skip degenerate triangles
        if v1 == v2 || v1 == v3 || v2 == v3 {
            continue;
        }

        // Map each vertex to a new compressed index
        let new_v1 = compressor.get_or_create_mapping(v1, || {
            let pos = mesh.vertex_position(&v1);
            FFIVector3::new(pos.x, pos.y, pos.z)
        });

        let new_v2 = compressor.get_or_create_mapping(v2, || {
            let pos = mesh.vertex_position(&v2);
            FFIVector3::new(pos.x, pos.y, pos.z)
        });

        let new_v3 = compressor.get_or_create_mapping(v3, || {
            let pos = mesh.vertex_position(&v3);
            FFIVector3::new(pos.x, pos.y, pos.z)
        });

        // Push directly to the output vector
        ffi_indices.push(new_v1);
        ffi_indices.push(new_v2);
        ffi_indices.push(new_v3);
    }

    // Get the final vertex array
    let ffi_vertices = compressor.vertices;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert("mesh.format".to_string(), "triangulated".to_string());
    let _ = return_config.insert("REMOVE_DOUBLES".to_string(), "false".to_string());

    Ok((ffi_vertices, ffi_indices, world_matrix, return_config))
}
