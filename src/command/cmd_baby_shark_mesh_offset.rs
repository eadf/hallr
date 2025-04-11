// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, ffi, prelude::FFIVector3};
use baby_shark::{
    exports::nalgebra::Vector3,
    mesh::{polygon_soup::data_structure::PolygonSoup, traits::Mesh},
    voxel::prelude::{MarchingCubesMesher, MeshToVolume},
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
    // todo: actually use the matrices
    let world_matrix = model.world_orientation.to_vec();

    let input_mesh = {
        let vertex_soup: Vec<Vector3<f32>> = model
            .indices
            .iter()
            .map(|&index| model.vertices[index].into())
            .collect();

        PolygonSoup::from_vertices(vertex_soup)
    };

    let mut mesh_to_volume = MeshToVolume::default()
        .with_voxel_size(input_config.get_mandatory_parsed_option("VOXEL_SIZE", None)?);

    println!("Rust: Starting baby_shark::offset()");
    let start = Instant::now();
    let mesh_volume = mesh_to_volume.convert(&input_mesh).unwrap();
    let offset = mesh_volume.offset(input_config.get_mandatory_parsed_option("OFFSET_BY", None)?);
    let vertices = MarchingCubesMesher::default()
        .with_voxel_size(offset.voxel_size())
        .mesh(&offset);
    let mesh = PolygonSoup::from_vertices(vertices);
    println!(
        "Rust: baby_shark::offset() execution time {:?}",
        start.elapsed()
    );

    let ffi_vertices = if let Some(world_to_local) = model.get_world_to_local_transform()? {
        println!(
            "Rust: applying world-local transformation 1/{:?}",
            model.world_orientation
        );
        mesh.vertices()
            .map(|i| world_to_local((*mesh.vertex_position(&i)).into()))
            .collect::<Vec<FFIVector3>>()
    } else {
        println!(
            "Rust: *not* applying world-local transformation 1/{:?}",
            model.world_orientation
        );
        mesh.vertices()
            .map(|i| (*mesh.vertex_position(&i)).into())
            .collect::<Vec<FFIVector3>>()
    };

    let ffi_indices: Vec<usize> = (0..mesh.vertices().count()).collect();

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
