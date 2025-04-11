// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

//#[cfg(test)]
//mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, ffi, prelude::FFIVector3};
use baby_shark::{
    exports::nalgebra::Vector3,
    mesh::polygon_soup::data_structure::PolygonSoup,
    voxel::prelude::{MarchingCubesMesher, MeshToVolume},
};
use hronn::HronnError;
use std::time::Instant;

pub(crate) fn process_command(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    if models.len() != 2 {
        Err(HronnError::InvalidParameter(
            "Incorrect number of models selected".to_string(),
        ))?
    }

    // todo: actually use the matrices
    let world_matrix = models[0].world_orientation.to_vec();

    let voxel_size = input_config.get_mandatory_parsed_option("voxel_size", None)?;
    let swap = input_config.get_mandatory_parsed_option("swap", Some(false))?;

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Triangulated)?;
    input_config.confirm_mesh_packaging(1, ffi::MeshFormat::Triangulated)?;

    let mut mesh_0_volume = {
        println!(
            "Rust: model0: {} vertices, {} indices",
            models[0].vertices.len(),
            models[0].indices.len()
        );
        let vertex_soup: Vec<Vector3<f32>> = models[0]
            .indices
            .iter()
            .map(|&index| models[0].vertices[index].into())
            .collect();
        let vertex_soup = PolygonSoup::from_vertices(vertex_soup);
        MeshToVolume::default()
            .with_voxel_size(voxel_size)
            .convert(&vertex_soup)
            .ok_or_else(|| {
                HallrError::InternalError("Baby Shark returned no volume for model 0".to_string())
            })?
    };

    let mut mesh_1_volume = {
        println!(
            "Rust: model1: {} vertices, {} indices",
            models[1].vertices.len(),
            models[1].indices.len()
        );
        let vertex_soup: Vec<Vector3<f32>> = models[1]
            .indices
            .iter()
            .map(|&index| models[1].vertices[index].into())
            .collect();
        let vertex_soup = PolygonSoup::from_vertices(vertex_soup);
        MeshToVolume::default()
            .with_voxel_size(voxel_size)
            .convert(&vertex_soup)
            .ok_or_else(|| {
                HallrError::InternalError("Baby Shark returned no volume for model 1".to_string())
            })?
    };

    if swap {
        std::mem::swap(&mut mesh_0_volume, &mut mesh_1_volume);
    }
    let operation = input_config.get_mandatory_option("operation")?;

    println!("Rust: Starting baby_shark::boolean()");
    let start = Instant::now();
    let return_vertices = {
        let volume = match operation {
            "DIFFERENCE" => mesh_0_volume.subtract(mesh_1_volume),
            "UNION" => mesh_0_volume.union(mesh_1_volume),
            "INTERSECT" => mesh_0_volume.intersect(mesh_1_volume),
            _ => Err(HallrError::InvalidParameter(
                format!("Invalid option: {}", operation).to_string(),
            ))?,
        };
        MarchingCubesMesher::default()
            .with_voxel_size(volume.voxel_size())
            .mesh(&volume)
    };
    println!(
        "Rust: baby_shark::boolean() execution time {:?}",
        start.elapsed()
    );
    let ffi_indices: Vec<usize> = (0..return_vertices.len()).collect();
    let ffi_vertices = return_vertices
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<FFIVector3>>();

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    // we take the easy way out here, and let blender do the de-duplication of the vertices.
    let _ = return_config.insert("REMOVE_DOUBLES".to_string(), "true".to_string());
    if let Some(value) = input_config.get("REMOVE_DOUBLES_THRESHOLD") {
        let _ = return_config.insert("REMOVE_DOUBLES_THRESHOLD".to_string(), value.clone());
    }

    Ok((ffi_vertices, ffi_indices, world_matrix, return_config))
}
