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
        // using the fast/unsafe variant
        IsotropicRemesh::<f32, _, true>::new(model.vertices, model.indices)
    })?;

    println!("Rust: Starting remesh()");
    let (mut ffi_vertices, ffi_indices) = time_it_r("Rust: remesh() took", || {
        //let remesher = remesher.with_print_stats(26)?;
        let remesher = remesher.with_target_edge_length(
            input_config.get_mandatory_parsed_option("TARGET_EDGE_LENGTH", None)?,
        )?;
        let remesher = match input_config.get_optional_parsed_option::<bool>("SPLIT_EDGES") {
            Ok(Some(true)) => remesher.with_split_edges(SplitStrategy::DihedralAngle)?,
            _ => remesher.without_split_edges()?,
        };

        let remesher = match input_config
            .get_mandatory_parsed_option::<String>("COLLAPSE_EDGES", Some("Disabled".to_string()))?
            .to_uppercase()
            .as_str()
        {
            "QEM" => {
                if let Some(qem_threshold) =
                    input_config.get_optional_parsed_option::<f32>("COLLAPSE_QEM_THRESHOLD")?
                {
                    remesher.with_collapse_qem_threshold(qem_threshold)?
                } else {
                    remesher.with_collapse_edges(CollapseStrategy::Qem)?
                }
            }
            "DIHEDRAL" => remesher.with_collapse_edges(CollapseStrategy::DihedralAngle)?,
            unknown => {
                println!("Got COLLAPSE_EDGES=={unknown}, turning off edge_collapse()");
                remesher.without_collapse_edges()?
            }
        };

        let flip_strategy = input_config
            .get_mandatory_parsed_option::<String>("FLIP_EDGES", Some("Disabled".to_string()))?;
        let remesher = match flip_strategy.as_str() {
            "disabled" => remesher.with_flip_edges(FlipStrategy::Disabled)?,
            "valence" => remesher.with_flip_edges(FlipStrategy::Valence)?,
            "quality" => {
                let qw = input_config
                    .get_mandatory_parsed_option::<f32>("FLIP_QUALITY_THRESHOLD", None)?;
                remesher.with_flip_edges(FlipStrategy::quality(qw))?
            }
            _ => Err(HallrError::InvalidParameter(
                format!("Invalid 'FLIP_EDGES' parameter:{}", flip_strategy).to_string(),
            ))?,
        };

        let remesher = if let Ok(Some(smooth_weight)) =
            input_config.get_optional_parsed_option::<f32>("SMOOTH_WEIGHT")
        {
            remesher.with_smooth_weight(smooth_weight)?
        } else {
            remesher.without_smooth_weight()?
        };

        let remesher = if let Some(coplanar_threshold) =
            input_config.get_optional_parsed_option::<f32>("COPLANAR_ANGLE_THRESHOLD")?
        {
            remesher.with_coplanar_angle_threshold(coplanar_threshold)?
        } else {
            remesher.with_default_coplanar_threshold()?
        };

        let remesher = if let Some(crease_threshold) =
            input_config.get_optional_parsed_option::<f32>("CREASE_ANGLE_THRESHOLD")?
        {
            remesher.with_crease_angle_threshold(crease_threshold)?
        } else {
            remesher.with_default_crease_threshold()?
        };

        Ok(remesher
            .run(input_config.get_mandatory_parsed_option::<usize>("ITERATIONS_COUNT", None)?)?)
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
