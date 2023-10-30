#![deny(
    rust_2018_compatibility,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    non_camel_case_types,
    unused_parens,
    non_upper_case_globals,
    unused_qualifications,
    unused_results,
    unused_imports,
    unused_variables,
    bare_trait_objects,
    ellipsis_inclusive_range_patterns,
    elided_lifetimes_in_paths
)]
#![warn(clippy::explicit_into_iter_loop)]
//mod collision;
pub mod command;
pub mod ffi;
mod geo;

use hronn::prelude::*;

pub mod prelude {
    pub use crate::ffi::{FFIVector3, GeometryOutput, StringMap, process_geometry, free_process_results};
    pub use crate::HallrError;
}

#[derive(thiserror::Error, Debug)]
pub enum HallrError {
    #[error(transparent)]
    HronnErr(#[from] HronnError),

    #[error(transparent)]
    LinestringError(#[from] linestring::LinestringError),

    #[error(transparent)]
    SpaceInsertionError(#[from] spade::InsertionError),

    #[error("Could not parse float value.")]
    ParseFloatError,

    #[error("The vertex indices does not match {0}")]
    MismatchedIndex(String),

    #[error("Your line-strings are self-intersecting: {0}")]
    SelfIntersectingData(String),

    #[error("The input data is not 2D: {0}")]
    InputNotPLane(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Aabb error: {0}")]
    AabbError(String),

    #[error("Transform error: {0}")]
    TransformError(String),

    #[error("Invalid input data: {0}")]
    InvalidParameter(String),

    #[error("Missing input data: {0}")]
    NoData(String),

    #[error("Obj file not triangulated: {0}")]
    NotTriangulated(String),

    #[error("Missing parameter: {0}")]
    MissingParameter(String),

    #[error("Mismatched MeshAnalyzer: {0}")]
    Mismatch(String),

    #[error("Unknown error: {0}")]
    InternalError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
