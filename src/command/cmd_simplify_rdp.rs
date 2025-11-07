// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model, Options};
use crate::{ffi, prelude::*, utils::IndexDeduplicator};
use hronn::prelude::ConvertTo;
use linestring::{
    linestring_3d::LineString3,
    prelude::{divide_into_shapes, indexed_simplify_rdp_2d, indexed_simplify_rdp_3d},
};
use vector_traits::{
    num_traits::AsPrimitive,
    prelude::{Aabb3, GenericScalar, GenericVector2, GenericVector3, HasXY, HasXYZ, Plane},
};

#[cfg(test)]
mod tests;

/// reformat the input from FFIVector3 to <GenericVector3> vertices.
fn parse_input<T: GenericVector3>(
    model: &Model<'_>,
) -> Result<(Vec<T>, <T as GenericVector3>::Aabb), HallrError>
where
    FFIVector3: ConvertTo<T>,
{
    let mut converted_vertices = Vec::<T>::with_capacity(model.vertices.len());
    let mut aabb = <T as GenericVector3>::Aabb::default();
    for p in model.vertices.iter() {
        if !p.is_finite() {
            return Err(HallrError::InvalidInputData(format!(
                "Only valid coordinates are allowed ({},{},{})",
                p.x(),
                p.y(),
                p.z()
            )));
        } else {
            let p: T = p.to();
            aabb.add_point(p);
            converted_vertices.push(p)
        }
    }

    Ok((converted_vertices, aabb))
}

pub(crate) fn process_command<T>(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: GenericVector3,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    f32: AsPrimitive<T::Scalar>,
{
    let cmd_simplify_distance: T::Scalar =
        input_config.get_mandatory_parsed_option("simplify_distance", None)?;

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Edges)?;

    let simplify_in_3d = input_config
        .get_optional_parsed_option("simplify_3d")?
        .unwrap_or(false);
    let mut output_vertices = Vec::<FFIVector3>::default();
    let mut output_indices = Vec::<usize>::default();
    let output_matrix;
    if !models.is_empty() && !models[0].indices.is_empty() {
        let model = &models[0];
        output_indices.reserve(model.indices.len());
        output_matrix = model.world_orientation.to_vec();
        let (vertices, aabb) = parse_input(&models[0])?;
        let simplify_distance =
            (aabb.max() - aabb.min()).magnitude() * cmd_simplify_distance / 100.0.into();

        if simplify_in_3d {
            // in 3d mode
            let mut vdd = IndexDeduplicator::<FFIVector3>::with_capacity(model.indices.len());

            for line in divide_into_shapes(model.indices).0 {
                let simplified = indexed_simplify_rdp_3d(&vertices, &line, simplify_distance);

                for line in simplified.windows(2) {
                    output_indices
                        .push(vdd.get_index_or_insert(line[0], || vertices[line[0]].to())? as usize);
                    output_indices
                        .push(vdd.get_index_or_insert(line[1], || vertices[line[1]].to())? as usize);
                }
            }
            output_vertices = vdd.vertices;
        } else {
            // in 2d mode
            let mut vdd = IndexDeduplicator::<FFIVector3>::with_capacity(model.indices.len());
            let vertices_2d = vertices.copy_to_2d(Plane::XY);

            for line in divide_into_shapes(model.indices).0 {
                let simplified = indexed_simplify_rdp_2d(&vertices_2d, &line, simplify_distance);

                for line in simplified.windows(2) {
                    output_indices.push(vdd.get_index_or_insert(line[0], || {
                        vertices_2d[line[0]].to_3d(T::Scalar::ZERO).to()
                    })? as usize);
                    output_indices.push(vdd.get_index_or_insert(line[1], || {
                        vertices_2d[line[1]].to_3d(T::Scalar::ZERO).to()
                    })? as usize);
                }
            }
            output_vertices = vdd.vertices;
        }
        if let Some(world_to_local) = model.get_world_to_local_transform()? {
            println!(
                "Rust: applying world-local transformation 1/{:?}",
                model.world_orientation
            );
            // Transform to local
            output_vertices
                .iter_mut()
                .for_each(|v| *v = world_to_local(*v));
        } else {
            println!("Rust: *not* applying world-local transformation");
        }
    } else {
        output_matrix = vec![];
    }
    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Edges.to_string(),
    );
    if let Some(mv) = input_config.get_optional_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }
    println!(
        "simplify_rdp operation returning {} vertices, {} indices",
        output_vertices.len(),
        output_indices.len()
    );
    Ok((
        output_vertices,
        output_indices,
        output_matrix,
        return_config,
    ))
}
