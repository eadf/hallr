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
        let remesher = remesher
            .with_target_edge_length(
                input_config.get_mandatory_parsed_option("TARGET_EDGE_LENGTH", None)?,
            )?
            .with_iterations(
                input_config.get_mandatory_parsed_option::<usize>("ITERATIONS_COUNT", None)?,
            )?;
        let remesher = if let Ok(Some(edge_split)) =
            input_config.get_optional_parsed_option::<bool>("SPLIT_EDGES")
        {
            if edge_split {
                remesher.with_default_split_multiplier()?
            } else {
                remesher.without_split_multiplier()?
            }
        } else if let Some(edge_split) =
            input_config.get_optional_parsed_option::<f32>("SPLIT_EDGES")?
        {
            remesher.with_split_multiplier(edge_split)?
        } else {
            remesher
        };
        let remesher = if let Ok(Some(edge_split)) =
            input_config.get_optional_parsed_option::<bool>("COLLAPSE_EDGES")
        {
            if edge_split {
                remesher.with_default_collapse_multiplier()?
            } else {
                remesher.without_collapse_multiplier()?
            }
        } else if let Some(edge_split) =
            input_config.get_optional_parsed_option::<f32>("COLLAPSE_EDGES")?
        {
            remesher.with_collapse_multiplier(edge_split)?
        } else {
            remesher
        };

        let remesher = remesher.with_flip_edges(
            input_config.get_mandatory_parsed_option::<bool>("FLIP_EDGES", Some(false))?,
        )?;
        let remesher = if let Ok(Some(smooth_weight)) =
            input_config.get_optional_parsed_option::<bool>("SMOOTH_VERTICES")
        {
            if smooth_weight {
                remesher.with_default_smooth_weight()?
            } else {
                remesher.without_smooth_weight()?
            }
        } else if let Some(smooth_weight) =
            input_config.get_optional_parsed_option::<f32>("SMOOTH_VERTICES")?
        {
            remesher.with_smooth_weight(smooth_weight)?
        } else {
            remesher
        };

        let remesher = if let Some(coplanar_threshold) =
            input_config.get_optional_parsed_option::<f32>("COPLANAR_ANGLE_THRESHOLD")?
        {
            remesher.with_coplanar_angle_threshold(coplanar_threshold)?
        } else {
            remesher.with_default_coplanar_threshold()?
        };
        Ok(remesher.run()?)
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
