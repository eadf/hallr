// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model, Options};
use crate::{prelude::*, utils::IndexDeduplicator};
use hronn::prelude::ConvertTo;
use linestring::{
    linestring_3d::{Aabb3, LineString3, Plane},
    prelude::{divide_into_shapes, indexed_simplify_rdp_2d, indexed_simplify_rdp_3d},
};
use vector_traits::{
    num_traits::AsPrimitive, GenericScalar, GenericVector2, GenericVector3, HasXY, HasXYZ,
};

#[cfg(test)]
mod tests;

/// reformat the input from FFIVector3 to <GenericVector3> vertices.
fn parse_input<T: GenericVector3>(model: &Model<'_>) -> Result<(Vec<T>, Aabb3<T>), HallrError>
where
    FFIVector3: ConvertTo<T>,
{
    let mut converted_vertices = Vec::<T>::with_capacity(model.vertices.len());
    let mut aabb = Aabb3::<T>::default();
    for p in model.vertices.iter() {
        if !p.x().is_finite() || !p.y().is_finite() || !p.z().is_finite() {
            return Err(HallrError::InvalidInputData(format!(
                "Only valid coordinates are allowed ({},{},{})",
                p.x(),
                p.y(),
                p.z()
            )));
        } else {
            let p: T = p.to();
            aabb.update_with_point(p);
            converted_vertices.push(p)
        }
    }

    Ok((converted_vertices, aabb))
}

pub(crate) fn process_command<T: GenericVector3>(
    config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    f32: AsPrimitive<T::Scalar>,
{
    let cmd_simplify_distance: T::Scalar =
        config.get_mandatory_parsed_option("simplify_distance", None)?;
    //println!("rust: vertices.len():{}", vertices.len());
    //println!("rust: indices.len():{}", indices.len());
    //println!("rust: indices:{:?}", indices);
    //let result = divide_into_shapes(models[0].indices);
    //for group in result {
    //    println!("***group:{:?}", group);
    //}

    let simplify_in_3d = config.get_parsed_option("simplify_3d")?.unwrap_or(false);
    let mut output_vertices = Vec::<FFIVector3>::default();
    let mut output_indices = Vec::<usize>::default();
    let output_matrix;
    if !models.is_empty() && !models[0].indices.is_empty() {
        let model = &models[0];
        output_indices.reserve(model.indices.len());
        output_matrix = model.world_orientation.to_vec();
        let (vertices, aabb) = parse_input(&models[0])?;
        let simplify_distance = (aabb.get_high().unwrap() - aabb.get_low().unwrap()).magnitude()
            * cmd_simplify_distance
            / 100.0.into();

        if simplify_in_3d {
            // in 3d mode
            let mut vdd = IndexDeduplicator::<FFIVector3>::with_capacity(model.indices.len());

            for line in divide_into_shapes(model.indices) {
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

            for line in divide_into_shapes(model.indices) {
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
    } else {
        output_matrix = vec![];
    }
    let mut config = ConfigType::new();
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("REMOVE_DOUBLES".to_string(), "false".to_string());

    println!(
        "simplify_rdp operation returning {} vertices, {} indices",
        output_vertices.len(),
        output_indices.len()
    );
    Ok((output_vertices, output_indices, output_matrix, config))
}
