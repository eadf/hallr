// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use crate::{
    HallrError,
    command::{ConfigType, Model, Options},
    ffi,
};
use rayon::prelude::*;
use vector_traits::{
    glam::{self},
    prelude::{Aabb3, GenericVector3},
};

type Aabb3Type = <glam::Vec3 as GenericVector3>::Aabb;

/// returns a list of type-converted vertices, a list of edges, and an AABB padded by radius
#[allow(clippy::type_complexity)]
fn parse_input(
    model: &Model<'_>,
    cmd_arg_sdf_radius_multiplier: f32,
) -> Result<(Vec<(glam::Vec2, f32)>, Aabb3Type), HallrError> {
    let mut aabb = Aabb3Type::default();

    let vertices: Result<Vec<_>, HallrError> = model
        .vertices
        .iter()
        .map(|vertex| {
            if !vertex.is_finite() {
                Err(HallrError::InvalidInputData(format!(
                    "Only valid coordinates are allowed ({},{},{})",
                    vertex.x, vertex.y, vertex.z
                )))?
            } else {
                let (point2, radius) = (
                    glam::vec2(vertex.x, vertex.y),
                    vertex.z.abs() * cmd_arg_sdf_radius_multiplier,
                );
                let mut v_aabb = Aabb3Type::from_point(glam::vec3(point2.x, point2.y, 0.0));
                v_aabb.pad(glam::Vec3::splat(radius));
                aabb.add_aabb(&v_aabb);

                Ok((point2, radius))
            }
        })
        .collect();
    Ok((vertices?, aabb))
}

/// Run the voronoi_mesh command
pub(crate) fn process_command(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    if models.is_empty() {
        return Err(HallrError::InvalidInputData(
            "This operation requires ome input model".to_string(),
        ));
    }

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Edges)?;

    if models.len() > 1 {
        return Err(HallrError::InvalidInputData(
            "This operation only supports one model as input".to_string(),
        ));
    }

    let cmd_arg_sdf_divisions: f32 =
        input_config.get_mandatory_parsed_option("SDF_DIVISIONS", None)?;
    if !(9.9..600.1).contains(&cmd_arg_sdf_divisions) {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of SDF_DIVISIONS is [{}..{}[% :({})",
            10, 600, cmd_arg_sdf_divisions
        )));
    }

    let cmd_arg_sdf_radius_multiplier =
        input_config.get_mandatory_parsed_option::<f32>("SDF_RADIUS_MULTIPLIER", None)?;

    // we already tested a_command.models.len()
    let input_model = &models[0];

    println!("Rust: model.vertices:{:?}, ", input_model.vertices.len());

    let (vertices, aabb) = parse_input(input_model, cmd_arg_sdf_radius_multiplier)?;

    let (voxel_size, mesh) = crate::utils::rounded_cones_fsn::build_round_cones_voxel_mesh(
        cmd_arg_sdf_divisions,
        input_model.indices.par_chunks_exact(2).map(|i| {
            let e0 = vertices[i[0]];
            let e1 = vertices[i[1]];
            (
                glam::vec4(e0.0.x, e0.0.y, 0.0, e0.1 * cmd_arg_sdf_radius_multiplier),
                glam::vec4(e1.0.x, e1.0.y, 0.0, e1.1 * cmd_arg_sdf_radius_multiplier),
            )
        }),
        aabb,
    )?;

    let output_model = crate::utils::rounded_cones_fsn::build_output_model(
        Some(input_model),
        voxel_size,
        mesh,
        false,
    )?;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    if let Some(mv) = input_config.get_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }
    println!(
        "Rust: sdf mesh 2.5d operation returning {} vertices, {} indices",
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
