// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

//! This module contains the execution of the implemented commands.

mod cmd_2d_outline;
mod cmd_baby_shark_boolean;
mod cmd_baby_shark_decimate;
mod cmd_baby_shark_isotropic_remesh;
mod cmd_baby_shark_mesh_offset;
mod cmd_centerline;
mod cmd_convex_hull_2d;
#[cfg(feature = "generate_test_case_from_input")]
#[cfg(not(test))]
mod cmd_create_test;
mod cmd_delaunay_triangulation_2d;
mod cmd_discretize;
mod cmd_knife_intersect;
mod cmd_lsystems;
mod cmd_sdf_mesh_2_5_fsn;
mod cmd_sdf_mesh_2_5_saft;
mod cmd_sdf_mesh_fsn;
mod cmd_sdf_mesh_saft;
mod cmd_simplify_rdp;
pub mod cmd_surface_scan;
mod cmd_voronoi_diagram;
mod cmd_voronoi_mesh;

#[cfg(feature = "generate_test_case_from_input")]
#[cfg(not(test))]
mod cmd_wavefront_obj_logger;
mod trait_impl;

use crate::{ffi, ffi::FFIVector3, prelude::*};
use std::collections::HashMap;
use vector_traits::{
    approx::ulps_eq,
    glam::{Vec3A, Vec4Swizzles},
    prelude::{Affine3D, GenericVector3},
};

/// The largest dimension of the voronoi input, totally arbitrarily selected.
const DEFAULT_MAX_VORONOI_DIMENSION: f32 = 200000.0;

/// The length of one 'step' for curved edges discretization as a percentage of the longest
/// AABB axis of the object.
const DEFAULT_VORONOI_DISCRETE_DISTANCE: f32 = 0.0001;

type ConfigType = HashMap<String, String>;

const IDENTITY_MATRIX: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

#[cfg(feature = "generate_test_case_from_input")]
static TEST_CODE_GENERATION_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
#[cfg(feature = "generate_test_case_from_input")]
pub fn is_test_code_generation_enabled() -> bool {
    *TEST_CODE_GENERATION_ENABLED
        .get_or_init(|| std::env::var("HALLR_BUILD_TEST_FROM_INPUT").is_ok())
}
#[cfg(feature = "generate_test_case_from_input")]
static HALLR_DATA_LOGGER_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
#[cfg(feature = "generate_test_case_from_input")]
pub fn is_data_logger_enabled() -> bool {
    *HALLR_DATA_LOGGER_ENABLED.get_or_init(|| std::env::var("HALLR_DATA_LOGGER_PATH").is_ok())
}

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

    fn confirm_mesh_packaging(
        &self,
        model_nr: usize,
        expected_format: ffi::MeshFormat,
    ) -> Result<(), HallrError>;
}

/// A re-packaging of the input mesh, python still owns this data
/// The OwnedModel contains a similar type that contains data owned by rust.
pub struct Model<'a> {
    world_orientation: &'a [f32],
    vertices: &'a [FFIVector3],
    indices: &'a [usize],
}

impl Model<'_> {
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

    #[inline(always)]
    pub fn is_identity_matrix(matrix: &[f32]) -> bool {
        matrix.len() == 16
            && matrix
                .iter()
                .zip(&IDENTITY_MATRIX)
                .all(|(&a, &b)| ulps_eq!(a, b))
    }

    #[inline(always)]
    pub fn has_identity_orientation(&self) -> bool {
        Self::is_identity_matrix(self.world_orientation)
    }

    #[inline(always)]
    pub fn has_xy_transform_only(&self) -> bool {
        let matrix = self.world_orientation;
        // Check if array has correct size
        if matrix.len() != 16 {
            return false;
        }

        // Column-major format:
        // [0, 4, 8,  12]  // first column
        // [1, 5, 9,  13]  // second column
        // [2, 6, 10, 14]  // third column
        // [3, 7, 11, 15]  // fourth column

        // Z row must be [0, 0, scale_z, 0] where scale_z is typically 1
        // This means matrix[2], matrix[6], matrix[10], matrix[14] is [0, 0, scale_z, 0]

        // Check no X/Y rotations or shears affecting Z
        if !ulps_eq!(matrix[2], 0.0) || !ulps_eq!(matrix[6], 0.0) {
            return false;
        }

        // Check no Z translation
        if !ulps_eq!(matrix[14], 0.0) {
            return false;
        }

        // Check no perspective transformation
        if !ulps_eq!(matrix[3], 0.0)
            || !ulps_eq!(matrix[7], 0.0)
            || !ulps_eq!(matrix[11], 0.0)
            || !ulps_eq!(matrix[15], 1.0)
        {
            return false;
        }

        true
    }

    /// Returns a closure that transforms world coordinates back to local coordinates
    pub fn get_world_to_local_transform(
        &self,
    ) -> Result<Option<impl Fn(FFIVector3) -> FFIVector3>, HallrError> {
        use vector_traits::glam::{Mat4, Vec3, Vec4};

        if self.has_identity_orientation() {
            // Identity matrix - just return the vector unchanged
            return Ok(None);
        }

        // Convert the flat array to a glam Matrix4
        let world_matrix = {
            if self.world_orientation.len() != 16 {
                return Err(HallrError::InvalidInputData(
                    "The provided world orientation matrix was of the wrong size".to_string(),
                ));
            }

            // Construct a Mat4 from the slice
            Mat4::from_cols_array(<&[f32; 16]>::try_from(self.world_orientation)?)
        };

        // Calculate inverse matrix for the reverse transformation
        match <Mat4 as Affine3D>::try_inverse(&world_matrix) {
            Some(inverse_matrix) => {
                // Return closure that applies the inverse transform
                Ok(Some(move |v: FFIVector3| -> FFIVector3 {
                    let gv: Vec3 = v.into();
                    // Apply the inverse transformation to convert from world to local
                    (inverse_matrix * Vec4::new(gv.x, gv.y, gv.z, 1.0))
                        .xyz()
                        .into()
                }))
            }
            None => Err(HallrError::InvalidInputData(
                "World orientation matrix is not invertible".to_string(),
            )),
        }
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
        let vertices_key = format!("first_vertex_model_{model_counter}",);
        let (default_vertex_index, default_index) = if model_counter == 0 {
            (Some(0), Some(0))
        } else {
            (None, None)
        };
        // Check if the keys exist in the config
        if model_counter == 0 || config.does_option_exist(&vertices_key)? {
            if matrix.len() < 16 {
                // no matrix found for this model, consider model as non-existing
                break;
            }
            // Retrieve the vertex and index offset data as strings
            let vertices_idx: usize =
                config.get_mandatory_parsed_option(&vertices_key, default_vertex_index)?;
            let indices_idx: usize = config.get_mandatory_parsed_option(
                &format!("first_index_model_{model_counter}",),
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

    if matrix.len() % 16 != 0 {
        return Err(HallrError::InvalidInputData(
            "The matrix field must be a multiple of 16".to_string(),
        ));
    }

    validate_input_data::<T>(vertices, indices, &config)?;
    let models = collect_models::<T>(vertices, indices, matrix, &config)?;

    #[cfg(feature = "generate_test_case_from_input")]
    #[cfg(not(test))]
    {
        // We are placing these "fetures" behind an extra ENV feature gate to prevent accidental spam
        if is_test_code_generation_enabled() {
            // Used for debugging - records input data to help reproduce, and build tests cases from,
            // tricky edge cases
            cmd_create_test::process_command(&config, &models)?
        }
        if is_data_logger_enabled() {
            // Used for debugging - records input data to help reproduce tricky edge cases
            // This time the data is saved as .obj files
            cmd_wavefront_obj_logger::process_command(&config, &models)?;
        }
    }
    Ok(match config.get_mandatory_option(ffi::COMMAND_TAG)? {
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
        "sdf_mesh_2½_fsn" => cmd_sdf_mesh_2_5_fsn::process_command(config, models)?,
        "sdf_mesh_2½_saft" => cmd_sdf_mesh_2_5_saft::process_command(config, models)?,
        "sdf_mesh" => cmd_sdf_mesh_fsn::process_command(config, models)?,
        "sdf_mesh_saft" => cmd_sdf_mesh_saft::process_command(config, models)?,
        "discretize" => cmd_discretize::process_command(config, models)?,
        "baby_shark_decimate" => cmd_baby_shark_decimate::process_command(config, models)?,
        "baby_shark_isotropic_remesh" => {
            cmd_baby_shark_isotropic_remesh::process_command(config, models)?
        }
        "baby_shark_mesh_offset" => cmd_baby_shark_mesh_offset::process_command(config, models)?,
        "baby_shark_boolean" => cmd_baby_shark_boolean::process_command(config, models)?,
        "lsystems" => cmd_lsystems::process_command(config, models)?,
        illegal_command => Err(HallrError::InvalidParameter(format!(
            "Invalid command:{illegal_command}",
        )))?,
    })
}

#[cfg(test)]
fn test_3d_triangulated_mesh(result: &CommandResult) {
    result
        .3
        .confirm_mesh_packaging(0, ffi::MeshFormat::Triangulated)
        .unwrap();
    assert_eq!(result.1.len() % 3, 0);
    assert!(!result.1.is_empty());
    let number_of_vertices = result.0.len();
    assert!(number_of_vertices > 0);

    for t in result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }
}
