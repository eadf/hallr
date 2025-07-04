// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, ffi, utils::TimeKeeper};
use baby_shark::{
    mesh::{corner_table::CornerTableF, traits::FromIndexed},
    remeshing::incremental::IncrementalRemesher,
};
use dedup_mesh::prelude::*;
use hronn::HronnError;
use crate::ffi::FFIVector3;

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

    let target_edge_length =
        input_config.get_mandatory_parsed_option("TARGET_EDGE_LENGTH", None)?;
    let mut mesh = {
        let _ = TimeKeeper::new("Rust: building baby_shark CornerTableF");
        CornerTableF::from_vertex_and_face_iters(
            model.vertices.iter().map(|v| v.into()),
            model.indices.iter().copied(),
        )
    };

    {
        println!("Rust: Starting baby_shark::remesh()");
        let _ = TimeKeeper::new("Rust: baby_shark::remesh()");
        let remesher = IncrementalRemesher::new()
            .with_iterations_count(
                input_config.get_mandatory_parsed_option("ITERATIONS_COUNT", None)?,
            )
            .with_split_edges(
                input_config.get_mandatory_parsed_option::<bool>("SPLIT_EDGES", Some(false))?,
            )
            .with_collapse_edges(
                input_config.get_mandatory_parsed_option::<bool>("COLLAPSE_EDGES", Some(false))?,
            )
            .with_flip_edges(
                input_config.get_mandatory_parsed_option::<bool>("FLIP_EDGES", Some(false))?,
            )
            .with_shift_vertices(
                input_config.get_mandatory_parsed_option::<bool>("SHIFT_VERTICES", Some(false))?,
            )
            .with_project_vertices(
                input_config
                    .get_mandatory_parsed_option::<bool>("PROJECT_VERTICES", Some(false))?,
            );

        remesher.remesh(&mut mesh, target_edge_length)
    };

    // it would be nice with a reverse of the `CornerTableF::from_vertices_and_indices()` method here.
    let (mut ffi_vertices, ffi_indices) = {
        let _ = TimeKeeper::new("Rust: collecting baby_shark output data (+dedup)");
        dedup_exact_from_iter::<f32, usize, FFIVector3, Triangulated, CheckFinite, _, _>(
            mesh.faces().flat_map(|face_descriptor| {
                let face = mesh.face_vertices(face_descriptor);
                [face.0, face.1, face.2].into_iter()
            }),
            |i| *mesh.vertex_position(i),
            mesh.faces().count() * 3,
            PruneDegenerate,
        )?
    };

    if let Some(world_to_local) = model.get_world_to_local_transform()? {
        let _ = TimeKeeper::new(format!(
            "Rust: applying world-local transformation 1/{:?}",
            model.world_orientation
        ));
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
    //    // we take the easy way out here, and let blender do the de-duplication of the vertices.
    //    let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    //}

    Ok((ffi_vertices, ffi_indices, world_matrix, return_config))
}
