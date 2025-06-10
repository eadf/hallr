// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use crate::{
    HallrError,
    command::{ConfigType, Model, Options, OwnedModel},
    ffi,
    ffi::FFIVector3,
};

use saft::BoundingBox;
use std::time;

/// initialize the sdf capsules and generate the mesh
fn build_voxel(
    radius_multiplier: f32,
    divisions: f32,
    vertices: &[FFIVector3],
    edges: &[usize],
) -> Result<
    (
        f32, // <- voxel_size
        saft::TriangleMesh,
    ),
    HallrError,
> {
    use macaw;
    if vertices.len() >= u32::MAX as usize {
        return Err(HallrError::Overflow(format!(
            "Input data contains too many vertices. {}",
            vertices.len()
        )));
    }
    let mut aabb = BoundingBox::default();

    for v in vertices.iter() {
        aabb.extend(macaw::Vec3::new(v.x, v.y, v.z));
    }

    let dimensions = aabb.max - aabb.min;
    let max_dimension = dimensions.x.max(dimensions.y).max(dimensions.z);

    let scale = divisions / max_dimension;

    let mean_resolution = max_dimension * scale;
    let mesh_options = saft::MeshOptions {
        mean_resolution,
        max_resolution: mean_resolution,
        min_resolution: 8.0,
    };

    let now = time::Instant::now();
    let mut graph = saft::Graph::default();

    let capsules: Vec<_> = edges
        .chunks_exact(2)
        .filter_map(|e| {
            let v0 = vertices[e[0]];
            let v1 = vertices[e[1]];

            // Early check for zero radii before any expensive computations
            let z0_abs = v0.z.abs();
            let z1_abs = v1.z.abs();
            if z0_abs <= f32::EPSILON && z1_abs <= f32::EPSILON {
                None
            } else {
                // Only compute these if we know we'll use them
                let z0 = z0_abs * radius_multiplier * scale;
                let z1 = z1_abs * radius_multiplier * scale;
                let v0 = macaw::Vec3::new(v0.x * scale, v0.y * scale, 0.0);
                let v1 = macaw::Vec3::new(v1.x * scale, v1.y * scale, 0.0);
                Some(graph.tapered_capsule([v0, v1], [z0, z1]))
            }
        })
        .collect();

    let root = graph.op_union_multi(capsules);
    let mesh = saft::mesh_from_sdf(&graph, root, mesh_options)?;

    println!("Rust: mesh_from_sdf_saft() duration: {:?}", now.elapsed());
    Ok((1.0 / scale, mesh))
}

/// Build the return model, totally ignore colors
fn build_output_model(
    input_model: &Model<'_>,
    voxel_size: f32,
    mesh: saft::TriangleMesh,
) -> Result<OwnedModel, HallrError> {
    let vertices: Vec<FFIVector3> =
        if let Some(world_to_local) = input_model.get_world_to_local_transform()? {
            println!(
                "Rust: applying world-local transformation 1/{:?}",
                input_model.world_orientation
            );
            // Transform to local
            mesh.positions
                .iter()
                .map(|v| {
                    world_to_local(FFIVector3 {
                        x: voxel_size * v[0],
                        y: voxel_size * v[1],
                        z: voxel_size * v[2],
                    })
                })
                .collect()
        } else {
            println!("Rust: *not* applying world-local transformation");
            mesh.positions
                .iter()
                .map(|v| FFIVector3 {
                    x: voxel_size * v[0],
                    y: voxel_size * v[1],
                    z: voxel_size * v[2],
                })
                .collect()
        };

    Ok(OwnedModel {
        world_orientation: input_model.copy_world_orientation()?,
        vertices,
        indices: mesh.indices.into_iter().map(|i| i as usize).collect(),
    })
}

/// Run the sdf_mesh_saft command
pub(crate) fn process_command(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    if models.is_empty() {
        return Err(HallrError::InvalidInputData(
            "This operation requires one input model".to_string(),
        ));
    }

    if models.len() > 1 {
        return Err(HallrError::InvalidInputData(
            "This operation only supports one model as input".to_string(),
        ));
    }

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Edges)?;

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

    let (voxel_size, mesh) = build_voxel(
        cmd_arg_sdf_radius_multiplier,
        cmd_arg_sdf_divisions,
        input_model.vertices,
        input_model.indices,
    )?;

    let output_model = build_output_model(input_model, voxel_size, mesh)?;

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
        "SDF mesh saft operation returning {} vertices, {} indices",
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
