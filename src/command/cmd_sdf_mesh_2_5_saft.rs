// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

//#[cfg(test)]
//mod tests;

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
    verbose: bool,
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

    let radius = max_dimension * radius_multiplier; // unscaled
    let thickness = radius * 2.0; // unscaled
    let scale = divisions / max_dimension;

    if verbose {
        println!(
            "Voxelizing using tube thickness. {} = {}*{}*{}",
            thickness, max_dimension, radius_multiplier, scale
        );

        println!(
            "Voxelizing using divisions = {}, max dimension = {}, scale factor={} (max_dimension*scale={})",
            divisions,
            max_dimension,
            scale,
            max_dimension * scale
        );
        println!();

        println!("aabb.high:{:?}", aabb.max);
        println!("aabb.low:{:?}", aabb.min);
        println!("delta:{:?}", aabb.max - aabb.min);
    }
    let mean_resolution = max_dimension * scale;
    if verbose {
        println!("mean_resolution:{:?}", mean_resolution);
    }
    let mesh_options = saft::MeshOptions {
        mean_resolution,
        max_resolution: mean_resolution,
        min_resolution: 8.0,
    };

    let vertices: Vec<_> = vertices
        .iter()
        .map(|v| macaw::Vec3::new(v.x, v.y, v.z) * scale)
        .collect();

    // let radius = radius * scale; // now scaled
    let now = time::Instant::now();
    let mut graph = saft::Graph::default();

    let capsules: Vec<_> = edges
        .chunks_exact(2)
        .map(|e| {
            let mut v0 = vertices[e[0]];
            let mut v1 = vertices[e[1]];
            let z0 = v0.z.abs() * radius_multiplier;
            let z1 = v1.z.abs() * radius_multiplier;
            v0.z = 0.0;
            v1.z = 0.0;
            //tapered_capsule(&mut self, points: [Vec3; 2], radii: [f32; 2])
            graph.tapered_capsule([v0, v1], [z0, z1])
        })
        .collect();

    let root = graph.op_union_multi(capsules);
    let mesh = saft::mesh_from_sdf(&graph, root, mesh_options)?;

    if verbose {
        println!("mesh_from_sdf() duration: {:?}", now.elapsed());
    }
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
            println!(
                "Rust: *not* applying world-local transformation 1/{:?}",
                input_model.world_orientation
            );
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

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::LineChunks)?;

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

    println!("model.vertices:{:?}, ", input_model.vertices.len());

    let (voxel_size, mesh) = build_voxel(
        cmd_arg_sdf_radius_multiplier,
        cmd_arg_sdf_divisions,
        input_model.vertices,
        input_model.indices,
        true,
    )?;

    let output_model = build_output_model(input_model, voxel_size, mesh)?;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    let _ = return_config.insert("REMOVE_DOUBLES".to_string(), "true".to_string());
    if let Some(value) = input_config.get("REMOVE_DOUBLES_THRESHOLD") {
        let _ = return_config.insert("REMOVE_DOUBLES_THRESHOLD".to_string(), value.clone());
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
