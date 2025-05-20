// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

mod fast_surface_nets;
mod lsystems;
#[cfg(test)]
mod tests;

use ilattice::{glam as iglam, prelude::Extent};
use vector_traits::{
    glam::{Vec3, Vec4Swizzles},
    prelude::{Aabb3, GenericVector3},
};

use super::{ConfigType, Model, Options, OwnedModel};
use crate::{
    command::cmd_lsystems::lsystems::{Turtle, TurtleRules},
    ffi,
    ffi::MeshFormat,
    prelude::*,
};
use std::time;

/// remove empty space and comments
fn trim_lsystem_string(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for line in input.lines() {
        // Remove comments (everything after #)
        let line_without_comments = line.split('#').next().unwrap_or("");

        // Skip empty lines
        if line_without_comments.is_empty() {
            continue;
        }

        // Add the processed line to the result
        result.push_str(line_without_comments.trim());
        result.push('\n');
    }

    result
}

pub(crate) fn process_command(
    input_config: ConfigType,
    _models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    let processed_text = {
        let cmd_custom_turtle = input_config.get_mandatory_option("🐢")?;
        trim_lsystem_string(cmd_custom_turtle)
    };

    let output_matrix = Vec::<f32>::default();

    //println!("Trimmed_TURTLE:\n{}", processed_text);
    let now = time::Instant::now();
    let (result, dedup, sdf_divisions) = {
        let turtle_rules = TurtleRules::default().parse(&processed_text)?;
        let sdf_divisions = turtle_rules.get_sdf_divisions();
        let dedup = turtle_rules.get_dedup();
        (turtle_rules.exec(Turtle::default())?, dedup, sdf_divisions)
    };
    (!result.is_empty())
        .then_some(())
        .ok_or_else(|| HallrError::ParseError("Input did not generate any vertices".to_string()))?;

    //

    let aabb = {
        let mut aabb = <Vec3 as GenericVector3>::Aabb::default();
        for [p0, p1] in result.iter() {
            let mut aabb_point = <Vec3 as GenericVector3>::Aabb::from_point(p0.xyz());
            aabb_point.pad(Vec3::splat(p0.w));
            aabb.add_aabb(&aabb_point);

            let mut aabb_point = <Vec3 as GenericVector3>::Aabb::from_point(p1.xyz());
            aabb_point.pad(Vec3::splat(p1.w));
            aabb.add_aabb(&aabb_point);
        }
        aabb
    };
    println!("build_custom_turtle render() duration: {:?}", now.elapsed());

    println!("Turtle result: {aabb:?}");
    let mut return_config = ConfigType::new();

    let output_model = if let Some(_sdf_divisions) = sdf_divisions {
        let (min, _, shape) = aabb.extents();
        let extent = Extent::<iglam::Vec3A>::from_min_and_shape(
            iglam::vec3a(min.x, min.y, min.z),
            iglam::vec3a(shape.x, shape.y, shape.z),
        );

        let (voxel_size, mesh) =
            fast_surface_nets::build_voxel(_sdf_divisions as f32, result, extent)?;
        println!("mesh {:?}", mesh.len());
        let _ = return_config.insert(
            MeshFormat::MESH_FORMAT_TAG.to_string(),
            MeshFormat::Triangulated.to_string(),
        );
        fast_surface_nets::build_output_model(voxel_size, mesh, false)?
    } else {
        let mut output_vertices = Vec::<FFIVector3>::with_capacity(result.len());
        let mut output_indices = Vec::<usize>::with_capacity(result.len());

        //println!("results:");
        for [p0_4, p1_4] in result {
            let p0_3 = p0_4.xyz();
            let p1_3 = p1_4.xyz();
            //println!("found edge : {p0_4:?} - {p1_4:?}");

            output_indices.push(output_vertices.len());
            output_vertices.push(p0_3.into());
            output_indices.push(output_vertices.len());
            output_vertices.push(p1_3.into());
            //println!("edge from {} to {}", p0, p1);
        }
        println!("The aabb was : {aabb:?}");
        let _ = return_config.insert(
            MeshFormat::MESH_FORMAT_TAG.to_string(),
            MeshFormat::LineChunks.to_string(),
        );
        OwnedModel {
            vertices: output_vertices,
            indices: output_indices,
            world_orientation: OwnedModel::identity_matrix(),
        }
    };

    let dedup_value = dedup
        .filter(|&v| v > 0.0) // Only keep positive, > 0 dedup values
        .or_else(|| {
            input_config
                .get_parsed_option::<f64>(ffi::VERTEX_MERGE_TAG)
                .ok()?
        })
        .unwrap_or(0.0001);

    let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), dedup_value.to_string());

    Ok((
        output_model.vertices,
        output_model.indices,
        output_matrix,
        return_config,
    ))
}
