// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model, Options};
use crate::prelude::*;
use hronn::prelude::{ConvertTo, triangulate_vertices};

use crate::ffi;
use krakel::PointTrait;
use linestring::linestring_2d::{Aabb2, convex_hull};
use vector_traits::{GenericVector3, HasXY, num_traits::AsPrimitive};

#[cfg(test)]
mod tests;

fn aabb_delaunay_triangulation_2d<T>(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: GenericVector3,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
{
    let model = &models[0];
    let bounding_shape = &models[1];

    if bounding_shape.vertices.is_empty() {
        return Err(HallrError::NoData("The bounding box is empty".to_string()));
    }
    // compute the AABB of the bounding_vertices regardless of interconnection
    let aabb = {
        let mut aabb = Aabb2::<T::Vector2>::default();
        for v in bounding_shape.vertices {
            aabb.update_with_point(v.to().to_2d());
        }
        aabb
    };
    // Use the AABB to generate a convex hull
    let hull: Vec<T::Vector2> = aabb
        .convex_hull::<T::Vector2>()
        .unwrap_or(Vec::<T::Vector2>::default())
        .into_iter()
        //.map(|v| v.to_3d(T::Scalar::ZERO).to())
        .collect();

    let results = triangulate_vertices::<T, FFIVector3>(aabb, &hull, model.vertices)?;
    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    if let Some(mv) = input_config.get_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }
    Ok((
        results.0,
        results.1,
        model.world_orientation.to_vec(),
        return_config,
    ))
}

fn convex_hull_delaunay_triangulation_2d<T>(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: GenericVector3,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
{
    let model = &models[0];
    let bounding_shape = &models[1];

    // do not limit us to a line bound, - yet
    //let bounding_indices =
    //    crate::collision::continuous_loop_from_unordered_edges(bounding_indices)?;
    println!(
        "Rust: bounding_indices: {:?} bounding_vertices {:?}",
        bounding_shape.indices.len(),
        bounding_shape.vertices.len()
    );

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::PointCloud)?;
    input_config.confirm_mesh_packaging(1, ffi::MeshFormat::PointCloud)?;

    let convex_hull: Vec<T::Vector2> = {
        // strip the Z coordinate off the bounding shape
        let point_cloud: Vec<T::Vector2> = bounding_shape
            .vertices
            .iter()
            .map(|v| v.to().to_2d())
            .collect();
        convex_hull::graham_scan(&point_cloud)?
    };
    let aabb = Aabb2::with_points(&convex_hull);

    let results = triangulate_vertices::<T, FFIVector3>(aabb, &convex_hull, model.vertices)?;
    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    if let Some(mv) = input_config.get_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }
    Ok((
        results.0,
        results.1,
        model.world_orientation.to_vec(),
        return_config,
    ))
}

pub(crate) fn process_command<T>(
    config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: GenericVector3,
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
{
    if models.is_empty() {
        return Err(HallrError::NoData("No models found".to_string()));
    }
    if models.len() < 2 {
        return Err(HallrError::NoData("Bounding shape not found".to_string()));
    }

    match config.get_mandatory_option("bounds")? {
        "CONVEX_HULL" => convex_hull_delaunay_triangulation_2d::<T>(config, models),
        "AABB" => aabb_delaunay_triangulation_2d::<T>(config, models),
        bounds => Err(HallrError::InvalidParameter(format!(
            "{} is not a valid \"bounds\" parameter",
            bounds
        ))),
    }
}
