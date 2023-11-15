// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

//! This module contains the Rust to Python (or rather CTypes) interface
mod impls;

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    iter::successors,
    slice,
    time::Instant,
};
use vector_traits::glam::Vec2;

/// A simple 3D vector struct for FFI (Foreign Function Interface) usage.
///
/// This struct represents a 3D vector with `x`, `y`, and `z` components for FFI usage.
/// It's used to exchange data between Rust and other programming languages like C or Python.
///
/// # Example
///
/// ```
/// use hallr::prelude::FFIVector3;
///
/// // Create a new FFIVector3 instance
/// let vector = FFIVector3 { x: 1.0, y: 2.0, z: 3.0 };
///
/// // Perform operations with the vector
/// let result = vector.x + vector.y;
/// ```
#[derive(PartialEq, PartialOrd, Copy, Clone, Default)]
#[repr(C)]
pub struct FFIVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl FFIVector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn xy(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

/// A struct representing the geometry output for FFI (Foreign Function Interface) usage.
///
/// This struct is used to return geometry-related data from Rust to other programming languages
/// like C or Python via FFI. It includes information about vertices and indices.
///
/// # Fields
///
/// * `vertices`: A pointer to an array of `FFIVector3` representing vertices.
/// * `vertex_count`: The number of vertices in the geometry.
/// * `indices`: A pointer to an array of `usize` representing indices.
/// * `indices_count`: The number of indices in the geometry.
/// * `matrices`: A pointer to an array of `f32` representing world orientation (matrix)
/// * `matrices_count`: The number of elements (f32) in `matrices`,
#[repr(C)]
pub struct GeometryOutput {
    vertices: *mut FFIVector3,
    vertex_count: usize,
    indices: *mut usize,
    indices_count: usize,
    matrices: *mut f32,
    matrices_count: usize,
}

impl GeometryOutput {
    /// Deallocates the memory associated with the `GeometryOutput` vertices and indices.
    ///
    /// This method should be called to free the memory held by the `GeometryOutput`.
    /// It safely deallocates memory for both the vertices and indices, preventing memory
    /// leaks. This function is typically used in conjunction with the `free_process_results`
    /// function to release memory when it is no longer needed.
    ///
    /// # Safety
    /// This function uses unsafe Rust code to deallocate memory. It should only be
    /// called in situations where you are certain that the memory can be safely
    /// released.
    fn free(&self) {
        unsafe {
            // Convert the raw pointers back into Vecs, which will deallocate when dropped
            let _ = Vec::from_raw_parts(self.vertices, self.vertex_count, self.vertex_count);
            let _ = Vec::from_raw_parts(self.indices, self.indices_count, self.indices_count);
            let _ = Vec::from_raw_parts(self.matrices, self.matrices_count, self.matrices_count);
        }
    }
}

/// A struct representing a map of strings for FFI (Foreign Function Interface) usage.
///
/// This struct is used to pass a map of strings between Rust and other programming languages
/// like C or Python via FFI. It contains arrays of keys and values along with their counts.
///
/// # Fields
///
/// * `keys`: A pointer to an array of C-style strings (null-terminated character pointers) representing keys.
/// * `values`: A pointer to an array of C-style strings (null-terminated character pointers) representing values.
/// * `count`: The number of key-value pairs in the map.
#[repr(C)]
pub struct StringMap {
    keys: *mut *mut std::os::raw::c_char,
    values: *mut *mut std::os::raw::c_char,
    count: usize,
}

impl StringMap {
    /// Deallocates the memory associated with the `StringMap` keys and values.
    ///
    /// This method should be called to free the memory held by the `StringMap`. It
    /// safely deallocates memory for both the keys and values, preventing memory
    /// leaks. This function is typically used in conjunction with the
    /// `free_process_results` function to release memory when it is no longer needed.
    ///
    /// # Safety
    /// This function uses unsafe Rust code to deallocate memory. It should only be
    /// called in situations where you are certain that the memory can be safely
    /// released.
    fn free(&self) {
        unsafe {
            for i in 0..self.count {
                // Convert back to CString to free the memory
                let _ = CString::from_raw(*self.keys.add(i));
                let _ = CString::from_raw(*self.values.add(i));
            }

            // Convert the raw pointers back into Vecs, which will be dropped and deallocate memory
            let _keys_vec = Vec::from_raw_parts(self.keys, self.count, self.count);
            let _values_vec = Vec::from_raw_parts(self.values, self.count, self.count);
        }
    }
}

/// A struct representing the result of a process with geometry data and a string map for FFI (Foreign Function Interface) usage.
///
/// This struct is used to return the result of a process that involves geometry data and a string map
/// between Rust and other programming languages like C or Python via FFI.
///
/// # Fields
///
/// * `geometry`: The geometry output of the process, typically containing vertices and indices.
/// * `map`: A string map with key-value pairs that store additional information about the process.
///
#[repr(C)]
pub struct ProcessResult {
    pub geometry: GeometryOutput,
    pub map: StringMap,
}

/// Converts any Err object into a python side response.
fn process_command_error_handler(
    vertices: &[FFIVector3],
    indices: &[usize],
    matrix: &[f32],
    config: HashMap<String, String>,
) -> (
    Vec<FFIVector3>,
    Vec<usize>,
    Vec<f32>,
    HashMap<String, String>,
) {
    let start = Instant::now();
    let rv = match crate::command::process_command(vertices, indices, matrix, config) {
        Ok(rv) => rv,
        Err(err) => {
            eprintln!("{:?}", err);
            for cause in successors(Some(&err as &(dyn std::error::Error)), |e| e.source()) {
                eprintln!("Caused by: {:?}", cause);
            }
            let mut config = HashMap::new();
            let _ = config.insert("ERROR".to_string(), err.to_string());
            (vec![], vec![], vec![], config)
        }
    };
    let duration = start.elapsed();
    println!("Rust: Time elapsed in process_command() was {:?}", duration);
    rv
}

/// Processes the provided geometry (vertices and edges).
///
/// # Safety
///
/// This function is marked `unsafe` because it:
/// - Dereferences raw pointers that are passed in.
/// - Assumes the memory blocks pointed to by `input_vertices` and `input_edges` are valid and have sizes at least `vertex_count` and `edge_count` respectively.
/// - It's the caller's responsibility to ensure that the memory blocks are valid and can safely be accessed.
///
/// Furthermore, after using this function, you MUST NOT use the passed memory blocks from the caller's side until you're done with them in Rust, to avoid data races and undefined behavior.
///
/// For FFI purposes, the caller from other languages (like Python) must be aware of these safety requirements, even though they won't explicitly use `unsafe` in their language.
#[no_mangle]
pub unsafe extern "C" fn process_geometry(
    input_ffi_vertices: *const FFIVector3,
    vertex_count: usize,
    input_ffi_indices: *const usize,
    indices_count: usize,
    input_ffi_matrix: *const f32,
    matrix_count: usize,
    config: *const StringMap,
) -> ProcessResult {
    assert!(
        !config.is_null(),
        "Rust: process_geometry(): Config ptr was null"
    );
    let count = (*config).count;
    println!("Rust:Received config of size:{:?}", count);
    assert!(
        (*config).count < 1000,
        "Rust: process_geometry(): Number of configuration parameters was too large: {}",
        (*config).count
    );
    // Use (*config).keys and (*config).values to access the arrays.
    let keys = slice::from_raw_parts((*config).keys, count);
    let values = slice::from_raw_parts((*config).values, count);

    let mut input_config = HashMap::with_capacity(count);
    for i in 0..count {
        let key = CStr::from_ptr(*keys.get(i).unwrap())
            .to_str()
            .unwrap()
            .to_string();
        let value = CStr::from_ptr(*values.get(i).unwrap())
            .to_str()
            .unwrap()
            .to_string();
        // input_config now contains cloned strings.
        //println!("Rust:Received Key: {}, Value: {}", key, value);
        let _ = input_config.insert(key, value);
    }
    println!("Rust:Received config:{:?}", input_config);

    let input_vertices = slice::from_raw_parts(input_ffi_vertices, vertex_count);
    let input_indices = slice::from_raw_parts(input_ffi_indices, indices_count);
    let input_matrix = slice::from_raw_parts(input_ffi_matrix, matrix_count);
    println!("Rust:received {} vertices", input_vertices.len());
    println!("Rust:received {} indices", input_indices.len());
    println!("Rust:received {} matrix", input_matrix.len());

    let (output_vertices, output_indices, output_matrix, output_config) =
        process_command_error_handler(input_vertices, input_indices, input_matrix, input_config);
    println!(
        "Rust returning: vertices:{}, indices:{}, matrices:{}/16, config:{:?}",
        output_vertices.len(),
        output_indices.len(),
        output_matrix.len(),
        output_config
    );
    let rv_g = GeometryOutput {
        vertices: output_vertices.as_ptr() as *mut FFIVector3,
        vertex_count: output_vertices.len(),
        indices: output_indices.as_ptr() as *mut usize,
        indices_count: output_indices.len(),
        matrices: output_matrix.as_ptr() as *mut f32,
        matrices_count: output_matrix.len(),
    };

    // Convert the HashMap into two vectors of *mut c_char
    let mut output_keys = Vec::with_capacity(output_config.len());
    let mut output_values = Vec::with_capacity(output_config.len());

    for (k, v) in output_config.iter() {
        output_keys.push(CString::new(k.clone()).unwrap().into_raw());
        output_values.push(CString::new(v.clone()).unwrap().into_raw());
    }

    // Create the return map
    let rv_s = StringMap {
        keys: output_keys.as_ptr() as *mut *mut std::os::raw::c_char,
        values: output_values.as_ptr() as *mut *mut std::os::raw::c_char,
        count: output_config.len(),
    };

    let rv = ProcessResult {
        geometry: rv_g,
        map: rv_s,
    };

    // Prevent the vectors from being deallocated. Their memory is now owned by the caller until it
    // calls free_process_results on it.
    std::mem::forget(output_vertices);
    std::mem::forget(output_indices);
    std::mem::forget(output_matrix);
    std::mem::forget(output_keys);
    std::mem::forget(output_values);

    rv
}

/// Frees the memory associated with a `ProcessResult`.
///
/// This function releases the memory associated with the components of the `ProcessResult`
/// struct, including vertices, indices, and the StringMap. It is intended to be called
/// from Python after processing to ensure proper memory cleanup.
///
/// # Safety
/// This function should only be called with a valid pointer to a `ProcessResult` created
/// by the Rust code. Using it with an invalid or NULL pointer may lead to memory issues.
///
/// # Arguments
///
/// * `result` - A pointer to a `ProcessResult` struct that you want to free the memory for.
///
#[no_mangle]
pub unsafe extern "C" fn free_process_results(result: *mut ProcessResult) {
    assert!(
        !result.is_null(),
        "Rust: free_process_results(): result ptr was null"
    );
    println!(
        "Rust releasing memory: vertices:{}, indices:{}, matrices:{}, map items:{}",
        (*result).geometry.vertex_count,
        (*result).geometry.indices_count,
        (*result).geometry.matrices_count,
        (*result).map.count
    );
    (*result).geometry.free();
    (*result).map.free();
}
