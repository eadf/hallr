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
mod utils;

use centerline::CenterlineError;
use hronn::HronnError;

pub mod prelude {
    pub use crate::{
        ffi::{free_process_results, process_geometry, FFIVector3, GeometryOutput, StringMap},
        HallrError,
    };
}

#[derive(thiserror::Error, Debug)]
pub enum HallrError {
    #[error(transparent)]
    CenterlineError(#[from] CenterlineError),

    #[error(transparent)]
    HronnErr(#[from] HronnError),

    #[error(transparent)]
    LinestringError(#[from] linestring::LinestringError),

    #[error("Invalid input data: {0}")]
    InvalidParameter(String),

    #[error("Input model not in one plane or not crossing origin. {0}")]
    InputNotPLane(String),

    #[error("Invalid input data value: {0}")]
    InvalidInputData(String),

    #[error("Missing input data: {0}")]
    NoData(String),

    #[error("Missing parameter: {0}")]
    MissingParameter(String),

    #[error("Model must not contain any faces: {0}")]
    ModelContainsFaces(String),

    #[error("Unknown error: {0}")]
    InternalError(String),
}
