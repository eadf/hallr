mod convex_hull_2d;
mod delaunay_triangulation_2d;
mod simplify_rdp;
pub mod surface_scan;

use crate::{ffi::FFIVector3, prelude::*};
use std::collections::HashMap;
use vector_traits::glam::Vec3;

fn get_mandatory_numeric_option<'a, T: std::str::FromStr>(
    key: &'a str,
    map: &'a HashMap<String, String>,
) -> Result<T, HallrError> {
    match map.get(key) {
        Some(v) => match v.parse() {
            Ok(val) => Ok(val),
            Err(_) => Err(HallrError::InvalidParameter(format!(
                "Invalid value for parameter \"{}\": \"{}\"",
                key, v
            ))),
        },
        None => Err(HallrError::MissingParameter(
            format!("The parameter \"{key}\" was missing").to_string(),
        )),
    }
}

fn get_mandatory_bool_option<'a>(
    key: &'a str,
    map: &'a HashMap<String, String>,
) -> Result<bool, HallrError> {
    match map.get(key) {
        Some(v) => {
            let lowercase_v = v.to_lowercase();
            match lowercase_v.parse() {
                Ok(val) => Ok(val),
                Err(_) => Err(HallrError::InvalidParameter(format!(
                    "Invalid value for parameter \"{}\":\"{}\"",
                    key, v
                ))),
            }
        }
        None => Err(HallrError::MissingParameter(
            format!("The parameter \"{key}\" was missing").to_string(),
        )),
    }
}

fn get_mandatory_option<'a>(
    key: &str,
    map: &'a HashMap<String, String>,
) -> Result<&'a str, HallrError> {
    match map.get(key) {
        Some(v) => Ok(v),
        None => Err(HallrError::MissingParameter(
            format!("The parameter \"{key}\" was missing").to_string(),
        )),
    }
}

fn does_option_exist(key: &str, map: &HashMap<String, String>) -> Result<bool, HallrError> {
    match map.get(key) {
        Some(_) => Ok(true),
        _ => Ok(false),
    }
}

type ConfigType = HashMap<String, String>;

pub(crate) fn process_command(
    vertices: &[FFIVector3],
    indices: &[usize],
    config: ConfigType,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError> {
    // only surface scan is defined for now
    Ok(match get_mandatory_option("command", &config)? {
        "surface_scan" => {
            surface_scan::process_command::<Vec3, FFIVector3>(vertices, indices, config)?
        }
        "convex_hull_2d" => {
            convex_hull_2d::process_command::<Vec3, FFIVector3>(vertices, indices, config)?
        }
        "simplify_rdp" => {
            simplify_rdp::process_command::<Vec3, FFIVector3>(vertices, indices, config)?
        }
        "2d_delaunay_triangulation" => delaunay_triangulation_2d::process_command::<
            Vec3,
            FFIVector3,
        >(vertices, indices, config)?,
        illegal_command => Err(HallrError::InvalidParameter(format!(
            "Invalid command:{}",
            illegal_command
        )))?,
    })
}
