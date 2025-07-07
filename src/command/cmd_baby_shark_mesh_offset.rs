// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, ffi, ffi::FFIVector3, utils::time_it};
use baby_shark::{
    exports::nalgebra::Vector3,
    mesh::polygon_soup::data_structure::PolygonSoup,
    voxel::prelude::{MarchingCubesMesher, MeshToVolume},
};
use dedup_mesh::{CheckFinite, PruneDegenerate, Triangulated, dedup_exact_from_iter};
use hronn::HronnError;

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

    let input_mesh = time_it("Rust: building baby_shark PolygonSoup", || {
        let vertex_soup: Vec<Vector3<f32>> = model
            .indices
            .iter()
            .map(|&index| model.vertices[index].into())
            .collect();

        PolygonSoup::from_vertices(vertex_soup)
    });

    let mut mesh_to_volume = MeshToVolume::default()
        .with_voxel_size(input_config.get_mandatory_parsed_option("VOXEL_SIZE", None)?);

    let mesh_volume = mesh_to_volume.convert(&input_mesh).unwrap();
    let offset = mesh_volume.offset(input_config.get_mandatory_parsed_option("OFFSET_BY", None)?);

    let (mut ffi_vertices, ffi_indices) = {
        let bs_vertices = time_it("Rust: running baby_shark::offset()", || {
            println!("Rust: Starting baby_shark::offset()");
            MarchingCubesMesher::default()
                .with_voxel_size(offset.voxel_size())
                .mesh(&offset)
        });

        time_it("Rust: collecting baby_shark output data (+dedup)", || {
            dedup_exact_from_iter::<f32, usize, FFIVector3, Triangulated, CheckFinite, _, _>(
                0..bs_vertices.len(),
                |i| bs_vertices[i],
                bs_vertices.len(),
                PruneDegenerate,
            )
        })?
    };

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
    };

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );

    Ok((ffi_vertices, ffi_indices, world_matrix, return_config))
}
