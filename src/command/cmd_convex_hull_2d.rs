// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model, OwnedModel};
use crate::{HallrError, command::Options, ffi, ffi::FFIVector3};
use hronn::prelude::ConvertTo;
use krakel::PointTrait;
use linestring::linestring_2d::convex_hull;
use vector_traits::{GenericScalar, GenericVector2, GenericVector3, approx::UlpsEq};

#[cfg(test)]
mod tests;

pub(crate) fn process_command<T>(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: GenericVector3,
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T::Scalar: UlpsEq,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
{
    if models.is_empty() {
        return Err(HallrError::InvalidInputData(
            "No models detected".to_string(),
        ));
    }
    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::PointCloud)?;

    let input_model = &models[0];
    // convert the input vertices to 2d point cloud
    let input: Vec<_> = input_model
        .vertices
        .iter()
        .map(|v| v.to().to_2d())
        .collect();
    // calculate the convex hull, and convert back to 3d FFIVector3 vertices
    let mut rv_model =
        OwnedModel::with_capacity(input_model.vertices.len(), input_model.indices.len());
    let all_indices: Vec<usize> = (0..input_model.vertices.len()).collect();

    if let Some(world_to_local) = input_model.get_world_to_local_transform()? {
        println!(
            "Rust: applying world-local transformation 1/{:?}",
            input_model.world_orientation
        );
        convex_hull::convex_hull_par(&input, &all_indices, 400)?
            .iter()
            .for_each(|i| {
                rv_model.push(world_to_local(
                    input_model.vertices[*i]
                        .to()
                        .to_2d()
                        .to_3d(T::Scalar::ZERO)
                        .to(),
                ))
            })
    } else {
        println!("Rust: *not* applying world-local transformation");
        convex_hull::convex_hull_par(&input, &all_indices, 400)?
            .iter()
            .for_each(|i| {
                rv_model.push(
                    input_model.vertices[*i]
                        .to()
                        .to_2d()
                        .to_3d(T::Scalar::ZERO)
                        .to(),
                )
            })
    }
    rv_model.close_loop();
    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::LineWindows.to_string(),
    );
    println!(
        "convex_hull_2d operation returning {} vertices",
        rv_model.indices.len()
    );
    Ok((
        rv_model.vertices,
        rv_model.indices,
        input_model.world_orientation.to_vec(),
        return_config,
    ))
}
