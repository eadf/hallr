// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, prelude::FFIVector3, utils::IndexCompressor};
use baby_shark::{
    decimation::{edge_decimation::ConstantErrorDecimationCriteria, prelude::EdgeDecimator},
    mesh::{corner_table::prelude::CornerTableF, traits::Mesh},
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

    let mut mesh = CornerTableF::from_vertex_and_face_iters(
        model.vertices.iter().map(|v| v.into()),
        model.indices.iter().copied(),
    );
    let decimation_criteria = ConstantErrorDecimationCriteria::new(
        config.get_mandatory_parsed_option("ERROR_THRESHOLD", None)?,
    );
    let mut decimator = EdgeDecimator::new()
        .decimation_criteria(decimation_criteria)
        .min_faces_count(Some(
            config.get_mandatory_parsed_option("MIN_FACES_COUNT", None)?,
        ));

    println!("Rust: Starting baby_shark::decimate()");
    let start = Instant::now();
    decimator.decimate(&mut mesh);
    println!(
        "Rust: baby_shark::decimate() execution time {:?}",
        start.elapsed()
    );

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
    if let Some(value) = config.get("REMOVE_DOUBLES_THRESHOLD") {
        let _ = return_config.insert("REMOVE_DOUBLES_THRESHOLD".to_string(), value.clone());
    }

    Ok((ffi_vertices, ffi_indices, world_matrix, return_config))
}
