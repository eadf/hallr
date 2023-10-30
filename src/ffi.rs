mod impls;

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    iter::successors,
    os::raw::c_char,
    slice,
    time::Instant,
};
use vector_traits::glam::Vec2;

#[derive(PartialEq, PartialOrd, Copy, Clone, Default)]
#[repr(C)]
pub struct FFIVector3 {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
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

#[repr(C)]
pub struct GeometryOutput {
    vertices: *mut FFIVector3,
    vertex_count: usize,
    indices: *mut usize,
    indices_count: usize,
}

impl GeometryOutput {
    fn free(&self) {
        unsafe {
            // Convert the raw pointers back into Vecs, which will deallocate when dropped
            let _vertices =
                Vec::from_raw_parts(self.vertices, self.vertex_count, self.vertex_count);
            let _indices =
                Vec::from_raw_parts(self.indices, self.indices_count, self.indices_count);
        }
    }
}

#[repr(C)]
pub struct StringMap {
    keys: *mut *mut c_char,
    values: *mut *mut c_char,
    count: usize,
}

impl StringMap {
    fn free(&self) {
        unsafe {
            #[allow(clippy::ptr_offset_with_cast)]
            for i in 0..self.count {
                // Convert back to CString to free the memory
                let _ = CString::from_raw(*self.keys.offset(i as isize));
                let _ = CString::from_raw(*self.values.offset(i as isize));
            }

            // Convert the raw pointers back into Vecs, which will be dropped and deallocate memory
            let _keys_vec = Vec::from_raw_parts(self.keys, self.count, self.count);
            let _values_vec = Vec::from_raw_parts(self.values, self.count, self.count);
        }
    }
}

#[repr(C)]
pub struct ProcessResult {
    pub geometry: GeometryOutput,
    pub map: StringMap,
}

/// Converts any Err object into a python side response.
fn process_command_error_handler(
    vertices: &[FFIVector3],
    indices: &[usize],
    config: HashMap<String, String>,
) -> (Vec<FFIVector3>, Vec<usize>, HashMap<String, String>) {
    let start = Instant::now();
    let rv = match crate::command::process_command(vertices, indices, config) {
        Ok(rv) => rv,
        Err(err) => {
            eprintln!("{:?}", err);
            for cause in successors(Some(&err as &(dyn std::error::Error)), |e| e.source()) {
                eprintln!("Caused by: {:?}", cause);
            }
            let mut config = HashMap::new();
            let _ = config.insert("ERROR".to_string(), err.to_string());
            (vec![], vec![], config)
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
    config: StringMap,
) -> ProcessResult {
    println!("Rust:Received config of size:{:?}", config.count);
    assert!(
        config.count < 1000,
        "process_geometry(): Number of configuration parameters was too large: {}",
        config.count
    );
    let mut input_config = HashMap::with_capacity(config.count);
    for i in 0..config.count {
        let key = CStr::from_ptr(*config.keys.add(i))
            .to_str()
            .unwrap()
            .to_string();
        let value = CStr::from_ptr(*config.values.add(i))
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
    println!("Rust:received {} vertices", input_vertices.len());
    println!("Rust:received {} indices", input_indices.len());

    let (output_vertices, output_indices, output_config) =
        process_command_error_handler(input_vertices, input_indices, input_config);
    println!(
        "Rust returning: vertices:{}, indices:{}, config:{:?}",
        output_vertices.len(),
        output_indices.len(),
        output_config
    );
    let rv_g = GeometryOutput {
        vertices: output_vertices.as_ptr() as *mut FFIVector3,
        vertex_count: output_vertices.len(),
        indices: output_indices.as_ptr() as *mut usize,
        indices_count: output_indices.len(),
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
        keys: output_keys.as_ptr() as *mut *mut c_char,
        values: output_values.as_ptr() as *mut *mut c_char,
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
    std::mem::forget(output_keys);
    std::mem::forget(output_values);

    rv
}

#[no_mangle]
pub extern "C" fn free_process_results(result: ProcessResult) {
    println!(
        "Rust releasing memory: vertices:{}, indices:{}, map items:{}",
        result.geometry.vertex_count, result.geometry.indices_count, result.map.count
    );
    result.geometry.free();
    result.map.free();
}
