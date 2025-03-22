// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

//! This module contains the execution of the implemented commands.

mod cmd_2d_outline;
mod cmd_baby_shark_decimate;
mod cmd_baby_shark_isotropic_remesh;
mod cmd_centerline;
mod cmd_convex_hull_2d;
mod cmd_delaunay_triangulation_2d;
mod cmd_discretize;
mod cmd_knife_intersect;
mod cmd_sdf_mesh;
mod cmd_sdf_mesh_2_5;
mod cmd_simplify_rdp;
pub mod cmd_surface_scan;
mod cmd_voronoi_diagram;
mod cmd_voronoi_mesh;
mod create_test;
mod impls;

use crate::{ffi::FFIVector3, prelude::*};
use std::collections::HashMap;
use vector_traits::{GenericVector3, approx::ulps_eq, glam::Vec3A};

/// The largest dimension of the voronoi input, totally arbitrarily selected.
const DEFAULT_MAX_VORONOI_DIMENSION: f32 = 200000.0;

/// The length of one 'step' for curved edges discretization as a percentage of the longest
/// AABB axis of the object.
const DEFAULT_VORONOI_DISCRETE_DISTANCE: f32 = 0.0001;

type ConfigType = HashMap<String, String>;

const IDENTITY_MATRIX: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

type CommandResult = (Vec<FFIVector3>, Vec<usize>, Vec<f32>, ConfigType);

trait Options {
    /// Will return an option parsed as a `T` or an Err
    fn get_mandatory_parsed_option<T: std::str::FromStr>(
        &self,
        key: &str,
        default: Option<T>,
    ) -> Result<T, HallrError>;

    /// Will return an option parsed as a `T` or None.
    /// If the option is missing None is returned, if it there but if it can't be parsed an error
    /// will be returned.
    fn get_parsed_option<T: std::str::FromStr>(&self, key: &str) -> Result<Option<T>, HallrError>;

    /// Returns the &str value of an option, or an Err is it does not exists
    fn get_mandatory_option(&self, key: &str) -> Result<&str, HallrError>;

    /// Returns true if the option exists
    fn does_option_exist(&self, key: &str) -> Result<bool, HallrError>;
}

/// A re-packaging of the input mesh, python still owns this data
pub struct Model<'a> {
    world_orientation: &'a [f32],
    vertices: &'a [FFIVector3],
    indices: &'a [usize],
}

impl<'a> Model<'a> {
    pub fn copy_world_orientation(&self) -> Result<[f32; 16], HallrError> {
        if self.world_orientation.len() == 16 {
            let mut rv = [0.0; 16];
            rv.copy_from_slice(self.world_orientation);
            return Ok(rv);
        }
        Err(HallrError::InvalidInputData(
            "The provided world orientation matrix was of the wrong size".to_string(),
        ))
    }

    pub fn is_identity_matrix(matrix: &[f32]) -> bool {
        matrix.len() == 16
            && matrix
                .iter()
                .zip(&IDENTITY_MATRIX)
                .all(|(&a, &b)| ulps_eq!(a, b))
    }

    pub fn has_identity_orientation(&self) -> bool {
        Self::is_identity_matrix(self.world_orientation)
    }
}

/// An owned variant of `Model`
pub struct OwnedModel {
    world_orientation: [f32; 16],
    vertices: Vec<FFIVector3>,
    indices: Vec<usize>,
}

impl OwnedModel {
    fn with_capacity(vertices_cap: usize, indices_cap: usize) -> Self {
        Self {
            world_orientation: [0.0; 16],
            vertices: Vec::<FFIVector3>::with_capacity(vertices_cap),
            indices: Vec::<usize>::with_capacity(indices_cap),
        }
    }

    #[allow(dead_code)]
    fn as_model(&self) -> Model<'_> {
        Model {
            world_orientation: &self.world_orientation,
            vertices: &self.vertices,
            indices: &self.indices,
        }
    }

    fn identity_matrix() -> [f32; 16] {
        IDENTITY_MATRIX
    }

    pub fn has_identity_orientation(&self) -> bool {
        Model::is_identity_matrix(&self.world_orientation)
    }

    /// Adds a point to the end of the list
    fn push(&mut self, value: FFIVector3) {
        self.indices.push(self.vertices.len());
        self.vertices.push(value);
    }

    /// close the loop by appending the first index last
    fn close_loop(&mut self) {
        if !self.indices.is_empty() {
            self.indices.push(*self.indices.first().unwrap())
        }
    }
}

/// Sanity check
pub fn validate_input_data<'a, T: GenericVector3>(
    vertices: &'a [FFIVector3],
    indices: &'a [usize],
    _config: &ConfigType,
) -> Result<(), HallrError> {
    if vertices.len() > u32::MAX as usize {
        Err(HallrError::InvalidInputData(
            "No more than u32::MAX vertices are supported.".to_string(),
        ))?
    }
    if indices.len() > u32::MAX as usize {
        Err(HallrError::InvalidInputData(
            "No more than u32::MAX indices are supported".to_string(),
        ))?
    }
    Ok(())
}

/// Collect the model data from `vertices`, `indices` and `config`
pub fn collect_models<'a, T: GenericVector3>(
    vertices: &'a [FFIVector3],
    indices: &'a [usize],
    mut matrix: &'a [f32],
    config: &ConfigType,
) -> Result<Vec<Model<'a>>, HallrError> {
    // Assuming you have a counter indicating the model number (0, 1, 2, ...)
    let mut models = Vec::new();
    let mut model_counter = 0;

    loop {
        // Construct the keys based on the model number
        let vertices_key = format!("first_vertex_model_{}", model_counter);
        let (default_vertex_index, default_index) = if model_counter == 0 {
            (Some(0), Some(0))
        } else {
            (None, None)
        };
        // Check if the keys exist in the config
        if model_counter == 0 || config.does_option_exist(&vertices_key)? {
            if matrix.len() < 16 {
                return Err(HallrError::InvalidInputData(
                    "World matrix data missing".to_string(),
                ));
            }
            // Retrieve the vertex and index data as strings
            let vertices_idx: usize =
                config.get_mandatory_parsed_option(&vertices_key, default_vertex_index)?;
            let indices_idx: usize = config.get_mandatory_parsed_option(
                &format!("first_index_model_{}", model_counter),
                default_index,
            )?;
            let vertices_end_idx: usize = config
                .get_parsed_option(&format!("first_vertex_model_{}", model_counter + 1))?
                .unwrap_or(vertices.len());
            let indices_end_idx: usize = config
                .get_parsed_option(&format!("first_index_model_{}", model_counter + 1))?
                .unwrap_or(indices.len());

            models.push(Model::<'_> {
                world_orientation: &matrix[0..16],
                vertices: &vertices[vertices_idx..vertices_end_idx],
                indices: &indices[indices_idx..indices_end_idx],
            });
            matrix = &matrix[16..];
            // Move on to the next model
            model_counter += 1;
        } else {
            // Break the loop when no more keys are found
            break;
        }
    }
    Ok(models)
}

/// This is the main FFI entry point, once the FFI module has sorted out all the messy c_ptr types
/// it will forward all request here.
pub(crate) fn process_command(
    vertices: &[FFIVector3],
    indices: &[usize],
    matrix: &[f32],
    config: ConfigType,
) -> Result<CommandResult, HallrError> {
    // the type we use for the internal processing
    type T = Vec3A;

    validate_input_data::<T>(vertices, indices, &config)?;
    let models = collect_models::<T>(vertices, indices, matrix, &config)?;

    if false {
        create_test::process_command(&config, &models)?
    }
    Ok(match config.get_mandatory_option("command")? {
        "surface_scan" => cmd_surface_scan::process_command::<T>(config, models)?,
        "convex_hull_2d" => cmd_convex_hull_2d::process_command::<T>(config, models)?,
        "simplify_rdp" => cmd_simplify_rdp::process_command::<T>(config, models)?,
        "2d_delaunay_triangulation" => {
            cmd_delaunay_triangulation_2d::process_command::<T>(config, models)?
        }
        "centerline" => cmd_centerline::process_command::<T>(config, models)?,
        "2d_outline" => cmd_2d_outline::process_command::<T>(config, models)?,
        "knife_intersect" => cmd_knife_intersect::process_command::<T>(config, models)?,
        "voronoi_mesh" => cmd_voronoi_mesh::process_command(config, models)?,
        "voronoi_diagram" => cmd_voronoi_diagram::process_command(config, models)?,
        "sdf_mesh_2_5" => cmd_sdf_mesh_2_5::process_command(config, models)?,
        "sdf_mesh" => cmd_sdf_mesh::process_command(config, models)?,
        "discretize" => cmd_discretize::process_command(config, models)?,
        "baby_shark_decimate" => cmd_baby_shark_decimate::process_command(config, models)?,
        "baby_shark_isotropic_remesh" => {
            cmd_baby_shark_isotropic_remesh::process_command(config, models)?
        }
        illegal_command => Err(HallrError::InvalidParameter(format!(
            "Invalid command:{}",
            illegal_command
        )))?,
    })
}
