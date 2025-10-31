// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{
    HallrError,
    command::Options,
    ffi,
    utils::{time_it, time_it_r},
};
use hronn::HronnError;
use remesh::prelude::*;

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

    let remesher = time_it("Rust: building IsotropicRemesh input", || {
        IsotropicRemesh::<f32, _, 0>::new(model.vertices, model.indices)
    })?;

    println!("Rust: Starting remesh()");
    let (mut ffi_vertices, ffi_indices) = time_it_r("Rust: remesh()", || {
        Ok(remesher
            .with_target_edge_length(
                input_config.get_mandatory_parsed_option("TARGET_EDGE_LENGTH", None)?,
            )?
            .with_iterations(
                input_config.get_mandatory_parsed_option::<usize>("ITERATIONS_COUNT", None)?,
            )?
            .with_split_multiplier(
                input_config.get_mandatory_parsed_option::<bool>("SPLIT_EDGES", Some(false))?,
            )?
            .with_collapse_multiplier(
                input_config.get_mandatory_parsed_option::<bool>("COLLAPSE_EDGES", Some(false))?,
            )?
            .with_flip_edges(
                input_config.get_mandatory_parsed_option::<bool>("FLIP_EDGES", Some(false))?,
            )?
            .with_smooth_weight(
                input_config.get_mandatory_parsed_option::<bool>("SMOOTH_VERTICES", Some(false))?,
            )?
            .run()?)
    })?;

    if let Some(world_to_local) = model.get_world_to_local_transform()? {
        time_it(
            format!(
                "Rust: applying world-local transformation 1/{:?}",
                model.world_orientation
            ),
            || {
                ffi_vertices
                    .iter_mut()
                    .for_each(|v| *v = world_to_local(*v));
            },
        )
    } else {
        println!("Rust: *not* applying world-local transformation");
    }

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );

    Ok((ffi_vertices, ffi_indices, world_matrix, return_config))
}
