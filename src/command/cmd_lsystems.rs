// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

mod lsystems;
#[cfg(test)]
mod tests;

use super::{ConfigType, Model, Options};
use crate::{
    command::cmd_lsystems::lsystems::{Turtle, TurtleRules},
    ffi::{MESH_FORMAT_TAG, MeshFormat},
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
    config: ConfigType,
    _models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    let processed_text = {
        let cmd_custom_turtle = config.get_mandatory_option("🐢")?;
        trim_lsystem_string(cmd_custom_turtle)
    };

    let output_matrix = Vec::<f32>::default();

    //println!("Trimmed_TURTLE:\n{}", processed_text);
    let now = time::Instant::now();
    let result = TurtleRules::default()
        .parse(&processed_text)?
        .exec(Turtle::default())?;
    (!result.is_empty())
        .then_some(())
        .ok_or_else(|| HallrError::ParseError("Input did not generate any vertices".to_string()))?;

    let mut output_vertices = Vec::<FFIVector3>::with_capacity(result.len());
    let mut output_indices = Vec::<usize>::with_capacity(result.len());

    // println!("Turtle result: {:?}", _result);

    //println!("results:");
    for [p0, p1] in result {
        output_vertices.push(FFIVector3::new(p0.x as f32, p0.y as f32, p0.z as f32));
        output_vertices.push(FFIVector3::new(p1.x as f32, p1.y as f32, p1.z as f32));
        //println!("edge from {} to {}", p0, p1);
        // Add indices for these two points (forming an edge)
        let base_index = output_vertices.len() - 2;
        output_indices.push(base_index);
        output_indices.push(base_index + 1);
    }
    /*
    println!("confirmation:");
    for edge in output_indices.chunks_exact(2) {
        println!("edge from {} to {}", output_vertices[edge[0]], output_vertices[edge[1]]);
    }*/

    println!("build_custom_turtle render() duration: {:?}", now.elapsed());

    let mut config = ConfigType::new();
    let _ = config.insert(
        MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );
    let _ = config.insert("REMOVE_DOUBLES".to_string(), "true".to_string());
    let _ = config.insert(
        "REMOVE_DOUBLES_THRESHOLD".to_string(),
        "0.00001".to_string(),
    );

    Ok((output_vertices, output_indices, output_matrix, config))
}
