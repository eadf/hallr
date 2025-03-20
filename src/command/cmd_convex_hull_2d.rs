// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model, OwnedModel};
use crate::{HallrError, ffi::FFIVector3};
use hronn::prelude::ConvertTo;
use krakel::PointTrait;
use linestring::linestring_2d::convex_hull;
use vector_traits::{GenericScalar, GenericVector2, GenericVector3, approx::UlpsEq};

#[cfg(test)]
mod tests;

pub(crate) fn process_command<T>(
    _config: ConfigType,
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
    let model = &models[0];
    // convert the input vertices to 2d point cloud
    let input: Vec<_> = model.vertices.iter().map(|v| v.to().to_2d()).collect();
    // calculate the convex hull, and convert back to 3d FFIVector3 vertices
    let mut rv_model = OwnedModel::with_capacity(model.vertices.len(), model.indices.len());
    let all_indices: Vec<usize> = (0..model.vertices.len()).collect();
    convex_hull::convex_hull_par(&input, &all_indices, 400)?
        .iter()
        .for_each(|i| rv_model.push(model.vertices[*i].to().to_2d().to_3d(T::Scalar::ZERO).to()));
    rv_model.close_loop();
    let mut config = ConfigType::new();
    let _ = config.insert("mesh.format".to_string(), "line_windows".to_string());
    println!(
        "convex_hull_2d operation returning {} vertices",
        rv_model.indices.len()
    );
    Ok((
        rv_model.vertices,
        rv_model.indices,
        model.world_orientation.to_vec(),
        config,
    ))
}
