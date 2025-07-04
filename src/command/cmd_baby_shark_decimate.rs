// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use super::{ConfigType, Model};
use crate::{HallrError, command::Options, ffi, utils::TimeKeeper};
use baby_shark::{
    decimation::{EdgeDecimator, edge_decimation::ConstantErrorDecimationCriteria},
    mesh::{corner_table::CornerTableF, traits::FromIndexed},
};
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

    let mut mesh = CornerTableF::from_vertex_and_face_iters(
        model.vertices.iter().map(|v| v.into()),
        model.indices.iter().copied(),
    );
    let decimation_criteria = ConstantErrorDecimationCriteria::new(
        input_config.get_mandatory_parsed_option("ERROR_THRESHOLD", None)?,
    );
    {
        println!("Rust: Starting baby_shark::decimate()");
        let _ = TimeKeeper::new("Rust: baby_shark::decimate()");
        let mut decimator = EdgeDecimator::new()
            .decimation_criteria(decimation_criteria)
            .min_faces_count(Some(
                input_config.get_mandatory_parsed_option("MIN_FACES_COUNT", None)?,
            ));

        decimator.decimate(&mut mesh);
    }

    // it would be nice with a reverse of the `CornerTableF::from_vertices_and_indices()` method here.
    let (mut ffi_vertices, ffi_indices) = {
        let _ = TimeKeeper::new("Rust: collecting baby_shark output data (+dedup)");

        dedup_mesh::dedup_exact_from_iter::<
            f32,
            usize,
            FFIVector3,
            dedup_mesh::Triangulated,
            dedup_mesh::CheckFinite,
            _,
            _,
        >(
            mesh.faces().flat_map(|face_descriptor| {
                let face = mesh.face_vertices(face_descriptor);
                [face.0, face.1, face.2].into_iter()
            }),
            |i| *mesh.vertex_position(i),
            mesh.faces().count() * 3,
            dedup_mesh::PruneDegenerate,
        )?
    };

    if let Some(world_to_local) = model.get_world_to_local_transform()? {
        // Transform to local
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

    Ok((ffi_vertices, ffi_indices, world_matrix, return_config))
}
