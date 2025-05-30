// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, ffi, utils::IndexCompressor};
use baby_shark::{
    decimation::{EdgeDecimator, edge_decimation::ConstantErrorDecimationCriteria},
    mesh::{corner_table::CornerTableF, traits::FromIndexed},
};
use hronn::HronnError;
use std::time::Instant;

pub(crate) fn process_command(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    if models.len() != 1 {
        Err(HronnError::InvalidParameter(
            "Incorrect number of models selected".to_string(),
        ))?
    }
    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Triangulated)?;
    let model = &models[0];
    let world_matrix = model.world_orientation.to_vec();

    let mut mesh = CornerTableF::from_vertex_and_face_iters(
        model.vertices.iter().map(|v| v.into()),
        model.indices.iter().copied(),
    );
    let decimation_criteria = ConstantErrorDecimationCriteria::new(
        input_config.get_mandatory_parsed_option("ERROR_THRESHOLD", None)?,
    );
    let mut decimator = EdgeDecimator::new()
        .decimation_criteria(decimation_criteria)
        .min_faces_count(Some(
            input_config.get_mandatory_parsed_option("MIN_FACES_COUNT", None)?,
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
        let (i0, i1, i2) = mesh.face_vertices(face_descriptor);

        // Skip degenerate triangles
        if i0 == i1 || i0 == i2 || i1 == i2 {
            continue;
        }

        // Map each vertex to a new compressed index
        let ni0 = compressor.get_or_create_mapping(i0, || mesh.vertex_position(i0).into());
        let ni1 = compressor.get_or_create_mapping(i1, || mesh.vertex_position(i1).into());
        let ni2 = compressor.get_or_create_mapping(i2, || mesh.vertex_position(i2).into());

        // Push directly to the output vector
        ffi_indices.push(ni0);
        ffi_indices.push(ni1);
        ffi_indices.push(ni2);
    }

    if let Some(world_to_local) = model.get_world_to_local_transform()? {
        // Transform to local
        println!(
            "Rust: applying world-local transformation 1/{:?}",
            model.world_orientation
        );
        compressor
            .vertices
            .iter_mut()
            .for_each(|v| *v = world_to_local(*v));
    } else {
        println!("Rust: *not* applying world-local transformation");
    }

    // Get the final vertex array
    let ffi_vertices = compressor.vertices;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    if let Some(mv) = input_config.get_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }

    Ok((ffi_vertices, ffi_indices, world_matrix, return_config))
}
