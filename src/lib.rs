// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

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

//! Experimental Blender addon written in Rust. This is a work in progress; expect API changes.
//!
//! Design guideline: The Python-Rust API is kept as simple as possible to avoid issues such as
//! memory leaks and dangling pointers. For the same reason, the API is stateless, ensuring that
//! everything needed for a specific operation is contained within that operation.

pub mod command;
pub mod ffi;
pub(crate) mod utils;
use centerline::CenterlineError;
use hronn::HronnError;

pub mod prelude {
    pub use crate::{
        HallrError,
        ffi::{FFIVector3, GeometryOutput, StringMap, free_process_results, process_geometry},
    };
}

#[derive(thiserror::Error, Debug)]
pub enum HallrError {
    #[error(transparent)]
    SliceError(#[from] std::array::TryFromSliceError),

    #[error(transparent)]
    EarcutrError(#[from] earcutr::Error),

    #[error(transparent)]
    BoostVoronoiError(#[from] boostvoronoi::BvError),

    #[error(transparent)]
    CenterlineError(#[from] CenterlineError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    SaftError(#[from] saft::Error),

    #[error(transparent)]
    HronnErr(#[from] HronnError),

    #[error(transparent)]
    LinestringError(#[from] linestring::LinestringError),

    #[error("Overflow error: {0}")]
    Overflow(String),

    #[error("Invalid float value: {0}")]
    FloatNotFinite(String),

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

    #[error("Unknown error: {0}")]
    LSystems3D(String),

    #[error("Could not parse L-Systems: {0}")]
    ParseError(String),
}
