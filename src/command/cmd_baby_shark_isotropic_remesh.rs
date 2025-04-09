// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, ffi, prelude::FFIVector3, utils::IndexCompressor};
use baby_shark::{
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

    let target_edge_length = config.get_mandatory_parsed_option("TARGET_EDGE_LENGTH", None)?;
    let mut mesh = CornerTableF::from_vertex_and_face_iters(
        model.vertices.iter().map(|v| v.into()),
        model.indices.iter().copied(),
    );

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

    println!("Rust: Starting baby_shark::remesh()");
    let start = Instant::now();
    remesher.remesh(&mut mesh, target_edge_length);
    println!(
        "Rust: baby_shark::remesh() execution time {:?}",
        start.elapsed()
    );

    // it would be nice with a reverse of the `CornerTableF::from_vertices_and_indices()` method here.

    // Compress the ranges of the indices to a minimum
    let mut compressor = IndexCompressor::with_capacity(mesh.vertices().count());

    // Extract the triangles with remapped indices
    let mut ffi_indices = Vec::with_capacity(mesh.faces().count() * 3);

    for face_descriptor in mesh.faces() {
        let (i0, i1, i2) = mesh.face_vertices(&face_descriptor);

        // Skip degenerate triangles
        if i0 == i1 || i0 == i2 || i1 == i2 {
            continue;
        }

        // Map each vertex to a new compressed index
        let v0 = compressor.get_or_create_mapping(i0, || {
            let pos = mesh.vertex_position(&i0);
            FFIVector3::new(pos.x, pos.y, pos.z)
        });

        let v1 = compressor.get_or_create_mapping(i1, || {
            let pos = mesh.vertex_position(&i1);
            FFIVector3::new(pos.x, pos.y, pos.z)
        });

        let v2 = compressor.get_or_create_mapping(i2, || {
            let pos = mesh.vertex_position(&i2);
            FFIVector3::new(pos.x, pos.y, pos.z)
        });

        // Push directly to the output vector
        ffi_indices.push(v0);
        ffi_indices.push(v1);
        ffi_indices.push(v2);
    }

    // Get the final vertex array
    let ffi_vertices = compressor.vertices;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    let _ = return_config.insert("REMOVE_DOUBLES".to_string(), "false".to_string());
    if let Some(value) = config.get("REMOVE_DOUBLES_THRESHOLD") {
        let _ = return_config.insert("REMOVE_DOUBLES_THRESHOLD".to_string(), value.clone());
    }

    Ok((ffi_vertices, ffi_indices, world_matrix, return_config))
}
