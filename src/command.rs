mod cmd_2d_outline;
mod cmd_centerline;
mod convex_hull_2d;
mod delaunay_triangulation_2d;
mod impls;
mod simplify_rdp;
pub mod surface_scan;

use crate::{ffi::FFIVector3, prelude::*};
use std::collections::HashMap;
use vector_traits::{glam::Vec3, GenericVector3};

/// The largest dimension of the voronoi input, totally arbitrarily selected.
const DEFAULT_MAX_VORONOI_DIMENSION: f64 = 200000.0;

trait Options {
    /// Will return an option parsed as a `T` or an Err
    fn get_mandatory_parsed_option<T: std::str::FromStr>(&self, key: &str)
        -> Result<T, HallrError>;

    /// Will return an option parsed as a `T` or None.
    /// If the option is missing None is returned, if it there but if it can't be parsed an error
    /// will be returned.
    fn get_parsed_option<T: std::str::FromStr>(&self, key: &str) -> Result<Option<T>, HallrError>;

    /// Returns the &str value of an option, or an Err is it does not exists
    fn get_mandatory_option(&self, key: &str) -> Result<&str, HallrError>;

    /// Returns true if the option exists
    fn does_option_exist(&self, key: &str) -> Result<bool, HallrError>;
}

type ConfigType = HashMap<String, String>;

/// A re-packaging of the input mesh, python still owns this data
pub struct Model<'a> {
    vertices: &'a [FFIVector3],
    indices: &'a [usize],
}

/// An owned variant of `Model`
pub struct OwnedModel {
    vertices: Vec<FFIVector3>,
    indices: Vec<usize>,
}

/// Sanity check
pub fn validate_input_data<'a, T: GenericVector3>(
    vertices: &'a [FFIVector3],
    indices: &'a [usize],
    config: &ConfigType,
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
    let _ = config.get_mandatory_parsed_option::<usize>("first_vertex_model_0")?;
    let _ = config.get_mandatory_parsed_option::<usize>("first_index_model_0")?;
    Ok(())
}

/// Collect the model data from `vertices`, `indices` and `config`
pub fn collect_models<'a, T: GenericVector3>(
    vertices: &'a [FFIVector3],
    indices: &'a [usize],
    config: &ConfigType,
) -> Result<Vec<Model<'a>>, HallrError> {
    // Assuming you have a counter indicating the model number (0, 1, 2, ...)
    let mut models = Vec::new();
    let mut model_counter = 0;

    loop {
        // Construct the keys based on the model number
        let vertices_key = format!("first_vertex_model_{}", model_counter);

        // Check if the keys exist in the config
        if config.does_option_exist(&vertices_key)? {
            // Retrieve the vertex and index data as strings
            let vertices_idx: usize = config.get_mandatory_parsed_option(&vertices_key)?;
            let indices_idx: usize = config
                .get_mandatory_parsed_option(&format!("first_index_model_{}", model_counter))?;
            let vertices_end_idx: usize = config
                .get_parsed_option(&format!("first_vertex_model_{}", model_counter + 1))?
                .unwrap_or(vertices.len());
            let indices_end_idx: usize = config
                .get_parsed_option(&format!("first_index_model_{}", model_counter + 1))?
                .unwrap_or(indices.len());

            models.push(Model::<'_> {
                vertices: &vertices[vertices_idx..vertices_end_idx],
                indices: &indices[indices_idx..indices_end_idx],
            });

            // Move on to the next model
            model_counter += 1;
        } else {
            // Break the loop when no more keys are found
            break;
        }
    }
    Ok(models)
}

/// This is the main FFI entry point, all commands will be routed through this API
pub(crate) fn process_command(
    vertices: &[FFIVector3],
    indices: &[usize],
    config: ConfigType,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError> {
    // the type we use for the internal processing
    type T = Vec3;

    validate_input_data::<T>(vertices, indices, &config)?;
    let models = collect_models::<T>(vertices, indices, &config)?;

    Ok(match config.get_mandatory_option("command")? {
        "surface_scan" => surface_scan::process_command::<T>(config, models)?,
        "convex_hull_2d" => convex_hull_2d::process_command::<T>(config, vertices, indices)?,
        "simplify_rdp" => simplify_rdp::process_command::<T>(config, models)?,
        "2d_delaunay_triangulation" => {
            delaunay_triangulation_2d::process_command::<T>(config, models)?
        }
        "centerline" => cmd_centerline::process_command::<T>(config, models)?,
        "2d_outline" => cmd_2d_outline::process_command::<T>(config, models)?,
        illegal_command => Err(HallrError::InvalidParameter(format!(
            "Invalid command:{}",
            illegal_command
        )))?,
    })
}
