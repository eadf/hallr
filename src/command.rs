mod centerline;
mod convex_hull_2d;
mod delaunay_triangulation_2d;
mod impls;
mod simplify_rdp;
pub mod surface_scan;

use crate::{ffi::FFIVector3, prelude::*};
use std::collections::HashMap;
use vector_traits::{glam::Vec3, HasXYZ};

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
pub struct Model<'a, MESH: HasXYZ> {
    vertices: &'a [MESH],
    indices: &'a [usize],
}

/// An owned variant of `Model`
pub struct OwnedModel<MESH: HasXYZ> {
    vertices: Vec<MESH>,
    indices: Vec<usize>,
}

/// This is the main FFI entry point, all commands will be routed through this API
pub(crate) fn process_command(
    vertices: &[FFIVector3],
    indices: &[usize],
    config: ConfigType,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError> {
    // The type of the data sent from blender, this memory is still owned by blender/python
    type MeshType = FFIVector3;
    // the type we use for the math
    type T = Vec3;

    let models = vec![Model { vertices, indices }];

    Ok(match config.get_mandatory_option("command")? {
        "surface_scan" => surface_scan::process_command::<T, MeshType>(vertices, indices, config)?,
        "convex_hull_2d" => {
            convex_hull_2d::process_command::<T, MeshType>(vertices, indices, config)?
        }
        "simplify_rdp" => simplify_rdp::process_command::<T, MeshType>(vertices, indices, config)?,
        "2d_delaunay_triangulation" => {
            delaunay_triangulation_2d::process_command::<T, MeshType>(vertices, indices, config)?
        }
        "centerline" => centerline::process_command::<T, MeshType>(models, config)?,
        illegal_command => Err(HallrError::InvalidParameter(format!(
            "Invalid command:{}",
            illegal_command
        )))?,
    })
}
