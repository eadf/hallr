"""
A simple script that calls the unified FFI interface of the .dll/.so/.dylib of hallr
"""
import ctypes
import platform

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

indices = [0, 1, 1, 2, 9, 10, 3, 4, 4, 5, 6, 7, 7, 8, 8, 9, 5, 6, 10, 11, 11, 12, 2, 3, 12, 0]
vertices = [Vector3(0.07700001,0.68200004,0.0), Vector3(0.24900001,0.68200004,0.0), Vector3(0.41820008,0.5503613,0.0),
Vector3(0.3927917,0.3409537,0.0), Vector3(0.35900003,0.324,0.0), Vector3(0.4220993,0.26076046,0.0),
Vector3(0.53755563,0.093555555,0.0), Vector3(0.60400003,0.0,0.0), Vector3(0.48700002,0.0,0.0),
Vector3(0.29636115,0.25348613,0.0), Vector3(0.17500001,0.296,0.0), Vector3(0.17500001,0.0,0.0), Vector3(0.07700001,0.0,0.0)]
config = {"first_index_model_0": "0", "SIMPLIFY": "false", "KEEP_INPUT": "true", "NEGATIVE_RADIUS": "true", "first_vertex_model_0": "0", "command": "centerline", "REMOVE_INTERNALS": "false", "ANGLE": "89.00000133828577", "DISTANCE": "0.004999999888241291", "WELD": "true", "mesh.format": "line_chunks"}

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

edges = [(output_indices[i], output_indices[i + 1]) for i in range(0, len(output_indices) - 1, 2)]
#for p in output_vertices:
#    print("v", p)

for i in edges:
    print("e:", i, end="")

for i in edges:
    print("e:", output_vertices[i[0]], output_vertices[i[1]], end="")

