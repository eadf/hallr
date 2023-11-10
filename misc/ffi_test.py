"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.

A simple script that calls the unified FFI interface of the .dll/.so/.dylib of hallr
"""
import ctypes
import platform

# 1. Sample zero-length inputs
vertices = []  # An empty list for vertices
indices = []   # An empty list for indices
config = {"key":"value"}  # A dictionary for config

# 2. Define the structures
class Vector3(ctypes.Structure):
    _fields_ = [("x", ctypes.c_float),
                ("y", ctypes.c_float),
                ("z", ctypes.c_float)]

class StringMap(ctypes.Structure):
    _fields_ = [("keys", ctypes.POINTER(ctypes.c_char_p)),
                ("values", ctypes.POINTER(ctypes.c_char_p)),
                ("count", ctypes.c_size_t)]

class GeometryOutput(ctypes.Structure):
    _fields_ = [("vertices", ctypes.POINTER(Vector3)),
                ("vertex_count", ctypes.c_size_t),
                ("indices", ctypes.POINTER(ctypes.c_size_t)),
                ("indices_count", ctypes.c_size_t)]

class ProcessResult(ctypes.Structure):
    _fields_ = [("geometry", GeometryOutput),
                ("map", StringMap)]


system = platform.system()
library_name = "libhallr.dylib"  # Default to macOS
if system == "Linux":
   library_name = "libhallr.so"
elif system == "Windows":
   library_name = "hallr.dll"

rust_lib = ctypes.cdll.LoadLibrary("./blender_addon_exported/lib/" + library_name)
rust_lib.process_geometry.argtypes = [ctypes.POINTER(Vector3), ctypes.c_size_t,
                                          ctypes.POINTER(ctypes.c_size_t), ctypes.c_size_t,
                                          ctypes.POINTER(StringMap)]

rust_lib.process_geometry.restype = ProcessResult
rust_lib.free_process_results.argtypes = [ctypes.POINTER(ProcessResult)]
rust_lib.free_process_results.restype = None

# Print function signature
print(f"Function Signature for rust_lib.process_geometry: {rust_lib.process_geometry.__name__}")
print(f"Argument Types: {rust_lib.process_geometry.argtypes}")
print(f"Return Type: {rust_lib.process_geometry.restype}")

# 3. Convert the zero-length data to a ctypes-friendly format
vertices_ptr = (Vector3 * len(vertices))(*vertices)
indices_ptr = (ctypes.c_size_t * len(indices))(*indices)

keys_list = list(config.keys())
values_list = list(config.values())
keys_array = (ctypes.c_char_p * len(keys_list))(*[k.encode('utf-8') for k in keys_list])
values_array = (ctypes.c_char_p * len(values_list))(*[v.encode('utf-8') for v in values_list])
map_data = StringMap(keys_array, values_array, len(keys_list))
print("python: keys_list.len()", len(keys_array))
print("python: values_array.len()", len(values_array))
print("python: len(keys_list)", len(keys_list))
print("python: map_data.keys:", map_data.keys)
print("python: map_data.values:", map_data.values)
print("python: map_data.count:", map_data.count)
# 4. Make the call to rust
rust_result = rust_lib.process_geometry(vertices_ptr, len(vertices), indices_ptr, len(indices), map_data)

# 5. Handle the results
output_vertices = [(vec.x, vec.y, vec.z) for vec in
                   (rust_result.geometry.vertices[i] for i in range(rust_result.geometry.vertex_count))]
output_indices = [rust_result.geometry.indices[i] for i in range(rust_result.geometry.indices_count)]

output_map = {}
for i in range(rust_result.map.count):
    key = ctypes.string_at(rust_result.map.keys[i]).decode('utf-8')
    value = ctypes.string_at(rust_result.map.values[i]).decode('utf-8')
    output_map[key] = value

# Print the results
print("Python received:", rust_result.geometry.vertex_count, "vertices")
print("Python received:", rust_result.geometry.indices_count, "indices")
print("Python received:", output_map)
