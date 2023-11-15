// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model, Options};
use crate::prelude::*;
use hronn::prelude::{triangulate_vertices, ConvertTo};

use krakel::PointTrait;
use linestring::linestring_2d::{convex_hull, Aabb2};
use vector_traits::{num_traits::AsPrimitive, GenericVector3, HasXY};

#[cfg(test)]
mod tests;

fn aabb_delaunay_triangulation_2d<T: GenericVector3>(
    _config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
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
    let mut config = ConfigType::new();
    let _ = config.insert("mesh.format".to_string(), "triangulated".to_string());
    Ok((
        results.0,
        results.1,
        model.world_orientation.to_vec(),
        config,
    ))
}

fn convex_hull_delaunay_triangulation_2d<T: GenericVector3>(
    _config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
{
    let model = &models[0];
    let bounding_shape = &models[1];

    // do not limit us to a line bound, - yet
    //let bounding_indices =
    //    crate::collision::continuous_loop_from_unordered_edges(bounding_indices)?;
    println!("bounding_indices {:?}", bounding_shape.indices.len());
    println!("bounding_vertices {:?}", bounding_shape.vertices.len());

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
    let _ = return_config.insert("mesh.format".to_string(), "triangulated".to_string());
    Ok((
        results.0,
        results.1,
        model.world_orientation.to_vec(),
        return_config,
    ))
}

pub(crate) fn process_command<T: GenericVector3>(
    config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
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
