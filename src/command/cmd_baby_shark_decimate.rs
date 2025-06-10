// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, ffi};
use baby_shark::{
    decimation::{EdgeDecimator, edge_decimation::ConstantErrorDecimationCriteria},
    mesh::{corner_table::CornerTableF, traits::FromIndexed},
};
use dedup_mesh::PruneDegenerateType::PruneDegenerate;
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
    let start = Instant::now();
    let deduplicated = dedup_mesh::dedup_exact_from_iter::<
        f32,
        dedup_mesh::Triangulated,
        dedup_mesh::CheckFinite,
        _,
        _,
    >(
        mesh.faces().flat_map(|face_descriptor| {
            let face = mesh.face_vertices(face_descriptor);
            [face.0, face.1, face.2].into_iter()
        }),
        |i| *mesh.vertex_position(i),
        mesh.faces().count() * 3,
        PruneDegenerate,
    )?;
    println!(
        "Rust: vertex_deduplication_exact() execution time {:?}",
        start.elapsed()
    );

    // Get the final vertex array
    let mut ffi_vertices = ffi::unsafe_convert_vec(deduplicated.0);

    if let Some(world_to_local) = model.get_world_to_local_transform()? {
        // Transform to local
        println!(
            "Rust: applying world-local transformation 1/{:?}",
            model.world_orientation
        );
        ffi_vertices
            .iter_mut()
            .for_each(|v| *v = world_to_local(*v));
    } else {
        println!("Rust: *not* applying world-local transformation");
    }

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    //if let Some(mv) = input_config.get_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
    //   // we take the easy way out here, and let blender do the de-duplication of the vertices.
    //    let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    //}

    Ok((ffi_vertices, deduplicated.1, world_matrix, return_config))
}
