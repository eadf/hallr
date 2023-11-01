import bpy
import os
import platform
from importlib import reload
import ctypes
import bmesh

# workaround for the "ImportError: attempted relative import with no known parent package" problem:
DEV_MODE = False  # Set this to False for distribution
HALLR_LIBRARY = None


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


def load_latest_dylib(prefix="libhallr_"):
    global HALLR_LIBRARY
    if DEV_MODE:
        # this will be find-and-replaced by the build script
        directory = "HALLR__TARGET_RELEASE"

        # List all files in the directory with the given prefix
        files = [f for f in os.listdir(directory) if
                 os.path.isfile(os.path.join(directory, f)) and f.startswith(prefix)]

        # Sort files by their modification time
        files.sort(key=lambda x: os.path.getmtime(os.path.join(directory, x)), reverse=True)

        # Load the latest .dylib, .dll, .so, whatever
        if files:
            latest_dylib = os.path.join(directory, files[0])
            print("Loading lib: ", latest_dylib)
            rust_lib = ctypes.cdll.LoadLibrary(latest_dylib)
        else:
            raise ValueError("Could not find the hallr runtime library!")

    else:  # release mode
        if HALLR_LIBRARY:
            return HALLR_LIBRARY

        system = platform.system()
        library_name = "libhallr.dylib"  # Default to macOS
        if system == "Linux":
            library_name = "libhallr.so"
        elif system == "Windows":
            library_name = "hallr.dll"
        module_dir = os.path.dirname(__file__)  # Get the directory of the Python module
        dylib_path = os.path.join(module_dir, 'lib', library_name)
        # print("trying to load:", dylib_path)
        # os.environ['DYLD_FALLBACK_LIBRARY_PATH'] = module_dir
        rust_lib = ctypes.cdll.LoadLibrary(dylib_path)

    rust_lib.process_geometry.argtypes = [ctypes.POINTER(Vector3), ctypes.c_size_t,
                                          ctypes.POINTER(ctypes.c_size_t), ctypes.c_size_t,
                                          ctypes.POINTER(StringMap)]
    rust_lib.process_geometry.restype = ProcessResult

    rust_lib.free_process_results.argtypes = [ctypes.POINTER(ProcessResult)]
    rust_lib.free_process_results.restype = None
    HALLR_LIBRARY = rust_lib
    return rust_lib


def ctypes_close_library(lib):
    if DEV_MODE:
        dlclose_func = ctypes.CDLL(None).dlclose
        dlclose_func.argtypes = [ctypes.c_void_p]
        dlclose_func.restype = ctypes.c_int
        dlclose_func(lib._handle)


def handle_new_object(mesh_obj):
    bpy.context.collection.objects.link(mesh_obj)

    # Optionally make the new object active
    bpy.context.view_layer.objects.active = mesh_obj

    # Ensure that we are in object mode
    bpy.ops.object.mode_set(mode='OBJECT')

    # Deselect all objects
    bpy.ops.object.select_all(action='DESELECT')

    # Select the newly created object
    mesh_obj.select_set(True)


def handle_triangle_mesh(vertices, indices):
    # Convert the indices to Blender's polygon format
    # Assuming indices are [0, 1, 2, 2, 3, 4, ...], where each set of 3 is a triangle
    polygons = [tuple(indices[i:i + 3]) for i in range(0, len(indices), 3)]

    # Check if polygons list isn't empty (to ensure it's not a line or other non-mesh shape)
    if polygons:
        # Create a new mesh
        mesh = bpy.data.meshes.new(name="New_Mesh")
        mesh.from_pydata(vertices, [], polygons)

        # Create a new object using the mesh and link it to the current collection
        mesh_obj = bpy.data.objects.new("New_Object", mesh)
        handle_new_object(mesh_obj)


""""
def handle_line_mesh(vertices, indices):
    # Convert the indices to Blender's edge format
    # Assuming indices are [0, 1, 2, 3, ...], where each set of 2 is a line
    edges = [tuple(indices[i:i + 2]) for i in range(0, len(indices), 2)]

    # Check if edges list isn't empty
    if edges:
        # Create a new mesh
        print("Python: creating a new line mesh from ", len(vertices), " vertices and ", len(edges), " edges")
        mesh = bpy.data.meshes.new(name="New_Line_Mesh")
        mesh.from_pydata(vertices, edges, [])

        # Create a new object using the mesh and link it to the current collection
        mesh_obj = bpy.data.objects.new("New_Line_Object", mesh)
        handle_new_object(mesh_obj)
    else:
        print("Python: Got no edges from rust")
"""


def handle_windows_line_new_object(vertices, indices):
    """"
    Convert the indices to Blender's edge format
    Slide over each vertex pair and create a new object.
    This function assumes that the line is in the ".window(2)" format,
    I.e. indices are [0, 1, 2, 3, ...], where [(1,2), (2,3),...] forms edges.
    """
    print("Python: received ", len(vertices), " vertices and ", len(indices), " indices")
    # Create edges from pairs of indices using windows(2)
    edges = [(indices[i], indices[i + 1]) for i in range(len(indices) - 1)]

    # Check if edges list isn't empty
    if edges:
        # Create a new mesh
        mesh = bpy.data.meshes.new(name="New_Line_Mesh")
        mesh.from_pydata(vertices, edges, [])

        # Create a new object using the mesh and link it to the current collection
        mesh_obj = bpy.data.objects.new("New_Line_Object", mesh)
        handle_new_object(mesh_obj)


def handle_chunks_line_new_object(vertices, indices):
    """
    Convert the indices to Blender's edge format
    Slide over each vertex pair and create a new object.
    This function assumes that the line is in the ".chunks(2)" format,
    i.e., indices are [0, 1, 2, 3, ...], where [(1,2), (3,4),...] forms edges.
    """
    print("Python: received ", len(vertices), " vertices and ", len(indices), " indices")

    # Convert the indices to Blender's edge format
    edges = [(indices[i], indices[i + 1]) for i in range(0, len(indices) - 1, 2)]

    # Check if the length is odd and print a warning
    if len(indices) % 2 != 0:
        print("Warning: Length of indices is odd. The last value may not form a valid edge pair.")
        print("indices:", indices)
        print("edges:", edges)

    # Check if edges list isn't empty
    if edges:
        # Create a new mesh
        mesh = bpy.data.meshes.new(name="New_Line_Mesh")
        mesh.from_pydata(vertices, edges, [])

        # Create a new object using the mesh and link it to the current collection
        mesh_obj = bpy.data.objects.new("New_Line_Object", mesh)
        handle_new_object(mesh_obj)


def handle_windows_line_modify_active_object(vertices, indices):
    """
    Convert vertices and indices to a bpy data, and insert into active object.
    This function assumes that the line is in the ".window(2)" format,
    I.e. indices are [0, 1, 2, 3, ...], where each set of 2 is a line
    """
    # Ensure that the active object is a mesh object
    active_obj = bpy.context.view_layer.objects.active
    if not active_obj or active_obj.type != 'MESH':
        print("No active mesh object to modify!")
        return

    # Convert the indices to Blender's edge format
    edges = [(indices[i], indices[i + 1]) for i in range(len(indices) - 1)]

    # Clear the existing geometry
    bpy.ops.object.mode_set(mode='EDIT')  # Must be in edit mode to use bmesh
    bm = bmesh.from_edit_mesh(active_obj.data)
    bm.clear()  # Clear all geometry

    # Create new vertices and edges
    verts = [bm.verts.new(vert) for vert in vertices]
    for edge in edges:
        bm.edges.new((verts[edge[0]], verts[edge[1]]))
    bmesh.update_edit_mesh(active_obj.data)  # Update the mesh with the changes

    bpy.ops.object.mode_set(mode='OBJECT')  # Switch back to object mode

    # Update the mesh
    active_obj.data.update()


def handle_chunks_line_modify_active_object(vertices, indices):
    """
    Convert vertices and indices to a bpy data, and insert into active object.
    This function assumes that the line is in the ".chunks(2)" format,
    i.e., indices are [0, 1, 2, 3, ...], where [(1,2), (3,4),...] forms edges.
    """
    # Ensure that the active object is a mesh object
    active_obj = bpy.context.view_layer.objects.active
    if not active_obj or active_obj.type != 'MESH':
        print("No active mesh object to modify!")
        return

    # Convert the indices to Blender's edge format
    edges = [(indices[i], indices[i + 1]) for i in range(0, len(indices) - 1, 2)]

    # Check if the length is odd and print a warning
    if len(indices) % 2 != 0:
        print("Warning: Length of indices is odd. The last value may not form a valid edge pair.")
        print("indices:", indices)
        print("edges:", edges)

    # Clear the existing geometry
    bpy.ops.object.mode_set(mode='EDIT')  # Must be in edit mode to use bmesh
    bm = bmesh.from_edit_mesh(active_obj.data)
    bm.clear()  # Clear all geometry

    # Create new vertices and edges
    verts = [bm.verts.new(vert) for vert in vertices]
    for edge in edges:
        bm.edges.new((verts[edge[0]], verts[edge[1]]))
    bmesh.update_edit_mesh(active_obj.data)  # Update the mesh with the changes

    bpy.ops.object.mode_set(mode='OBJECT')  # Switch back to object mode

    # Update the mesh
    active_obj.data.update()


def is_loop(mesh):
    """"
    Determines if a mesh is a loop of vertices
    """
    # Create a dictionary to store the count of edges connected to each vertex
    edge_count_per_vertex = {}

    for edge in mesh.edges:
        for vertex in edge.vertices:
            edge_count_per_vertex[vertex] = edge_count_per_vertex.get(vertex, 0) + 1
            # Check if the vertex is connected to more than two edges
            if edge_count_per_vertex[vertex] > 2:
                return False

    # Check if there's any vertex connected to just one edge
    if 1 in edge_count_per_vertex.values():
        return False

    # Check if the number of distinct vertices equals the number of edges
    return len(edge_count_per_vertex) == len(mesh.edges)


def has_un_applied_transformations(obj):
    """
    Returns true if an object has transformations
    """

    if obj.location.x != 0 or obj.location.y != 0 or obj.location.z != 0:
        return True
    if obj.rotation_mode == 'QUATERNION':
        # print("obj.rotation_quaternion", obj.rotation_quaternion)
        if obj.rotation_quaternion != (1, 0, 0, 0):
            return True
    elif obj.rotation_mode == 'AXIS_ANGLE':
        # This is a bit more complicated because it's represented with an angle and a vector
        # print("obj.rotation_axis_angle", obj.rotation_axis_angle)
        if obj.rotation_axis_angle[0] != 0:  # This checks the angle, which is the first element
            return True
    else:  # Euler
        # print("obj.rotation_euler", obj.rotation_euler)
        if obj.rotation_euler.x != 0 or obj.rotation_euler.y != 0 or obj.rotation_euler.z != 0:
            return True
    if obj.scale.x != 1 or obj.scale.y != 1 or obj.scale.z != 1:
        return True
    return False


def prepare_object_for_processing(obj, temp_name):
    """
    When applying transforms it's easier to do so on a copy of the original object.
    So this function duplicates the object and applies transformations if necessary.
    Returns the object to be processed and a flag indicating if it was duplicated.
    """
    # Deselect all objects
    bpy.ops.object.select_all(action='DESELECT')

    # Select only the object we want to duplicate
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj

    if has_un_applied_transformations(obj):
        # Duplicate the object
        bpy.ops.object.duplicate()
        dup_obj = bpy.context.object
        dup_obj.name = temp_name

        # Apply all transformations
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
        # print("object had transformations")
        return dup_obj, True
    return obj, False


def prepare_object_for_processing_direct(obj):
    if has_un_applied_transformations(obj):
        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    return obj


def call_rust(config: dict[str, str], active_obj, bounding_shape=None, only_selected_vertices=False):
    # Load the Rust library
    # We load the .dylib and define argtypes for every invocation just to be able to update the lib without
    # restarting blender. This does not seem to work anymore, though
    rust_lib = load_latest_dylib()

    # Prepare both objects for processing
    active_obj_to_process, active_obj_is_duplicated = prepare_object_for_processing(active_obj, "TempDuplicateActive")
    if not active_obj_to_process:
        raise RuntimeError("Error in finding the active mesh.")
    if bounding_shape:
        bounding_obj_to_process, bounding_obj_is_duplicated = prepare_object_for_processing(bounding_shape,
                                                                                            "TempDuplicateBounding")
        if not bounding_obj_to_process:
            raise RuntimeError("Error in finding the bounding shape.")

    if only_selected_vertices:
        indices = []
        vertices = [Vector3(v.co.x, v.co.y, v.co.z) for v in active_obj.data.vertices if v.select]
    else:
        # 4. Gather triangle vertex indices
        indices = [vert_idx for face in active_obj_to_process.data.polygons for vert_idx in face.vertices]

        # 5. Convert the data to a ctypes-friendly format
        vertices = [Vector3(v.co.x, v.co.y, v.co.z) for v in active_obj_to_process.data.vertices]

    # Keeping track of the current number of vertices before adding bounding shape
    start_vertex_index_for_bounding = len(vertices)

    if bounding_shape:
        # Appending vertices from the bounding shape
        vertices += [Vector3(v.co.x, v.co.y, v.co.z) for v in bounding_obj_to_process.data.vertices]

        # Take note of the starting index for the bounding shape in the indices list
        start_index_for_bounding = len(indices)

        config["start_vertex_index_for_bounding"] = str(start_vertex_index_for_bounding)
        config["start_index_for_bounding"] = str(start_index_for_bounding)

        # Appending edge vertex indices from the bounding shape, adjusting based on the start_vertex_index
        for edge in bounding_obj_to_process.data.edges:
            indices.append(edge.vertices[0])
            indices.append(edge.vertices[1])

    if active_obj_is_duplicated:
        cleanup_duplicated_object(active_obj_to_process)

    if bounding_shape and bounding_obj_is_duplicated:
        cleanup_duplicated_object(bounding_obj_to_process)

    # 6. Convert the data to a ctypes-friendly format
    vertices_ptr = (Vector3 * len(vertices))(*vertices)
    indices_ptr = (ctypes.c_size_t * len(indices))(*indices)

    # 7. Convert the dictionary to two separate lists for keys and values
    keys_list = list(config.keys())
    values_list = list(config.values())
    keys_array = (ctypes.c_char_p * len(keys_list))(*[k.encode('utf-8') for k in keys_list])
    values_array = (ctypes.c_char_p * len(values_list))(*[v.encode('utf-8') for v in values_list])
    map_data = StringMap(keys_array, values_array, len(keys_list))

    # 8. Make the call to rust
    rust_result = rust_lib.process_geometry(vertices_ptr, len(vertices), indices_ptr, len(indices), map_data)

    print("python received: ", rust_result.geometry.vertex_count, "vertices")
    print("python received: ", rust_result.geometry.indices_count, "indices")
    # 9. Handle the results
    output_vertices = [(vec.x, vec.y, vec.z) for vec in
                       (rust_result.geometry.vertices[i] for i in range(rust_result.geometry.vertex_count))]
    output_indices = [rust_result.geometry.indices[i] for i in range(rust_result.geometry.indices_count)]

    output_map = {}
    for i in range(rust_result.map.count):
        key = ctypes.string_at(rust_result.map.keys[i]).decode('utf-8')
        value = ctypes.string_at(rust_result.map.values[i]).decode('utf-8')
        output_map[key] = value
    print("python received: ", output_map)

    # 10. Free rust memory
    rust_lib.free_process_results(rust_result)

    # 11. try to close the .dylib so that it is "fresh" for the next invocation.
    # this does not seem to work anymore, though. It requires restart of blender to work
    ctypes_close_library(rust_lib)

    return output_vertices, output_indices, output_map


def call_rust_direct(config, active_obj, expect_line_chunks=False):
    """
    A simpler version of call_rust that only processes the active_object.
    When `expect_line_chunks` is set, the data will iterate over each edge(a,b) and use a list of
    indices in .chunks(2) format.
    """

    rust_lib = load_latest_dylib()

    active_obj_to_process = prepare_object_for_processing_direct(active_obj)
    vertices = [Vector3(v.co.x, v.co.y, v.co.z) for v in active_obj_to_process.data.vertices]
    vertices_ptr = (Vector3 * len(vertices))(*vertices)

    if expect_line_chunks:
        indices = [v for edge in active_obj_to_process.data.edges for v in edge.vertices]
    else:
        indices = [vert_idx for face in active_obj_to_process.data.polygons for vert_idx in face.vertices]

    indices_ptr = (ctypes.c_size_t * len(indices))(*indices)

    keys_list = list(config.keys())
    values_list = list(config.values())
    keys_array = (ctypes.c_char_p * len(keys_list))(*[k.encode('utf-8') for k in keys_list])
    values_array = (ctypes.c_char_p * len(values_list))(*[v.encode('utf-8') for v in values_list])
    map_data = StringMap(keys_array, values_array, len(keys_list))

    rust_result = rust_lib.process_geometry(vertices_ptr, len(vertices), indices_ptr, len(indices), map_data)

    output_vertices = [(vec.x, vec.y, vec.z) for vec in
                       (rust_result.geometry.vertices[i] for i in range(rust_result.geometry.vertex_count))]
    output_indices = [rust_result.geometry.indices[i] for i in range(rust_result.geometry.indices_count)]

    output_map = {}
    for i in range(rust_result.map.count):
        key = ctypes.string_at(rust_result.map.keys[i]).decode('utf-8')
        value = ctypes.string_at(rust_result.map.values[i]).decode('utf-8')
        output_map[key] = value

    rust_lib.free_process_results(rust_result)
    ctypes_close_library(rust_lib)

    return output_vertices, output_indices, output_map


def cleanup_duplicated_object(an_obj):
    """
    Deletes the duplicated object if it exists.
    """
    obj_name = an_obj.name
    if obj_name in bpy.data.objects:
        obj = bpy.data.objects[obj_name]
        if obj:
            # print(f"Trying to delete {obj_name} which is linked to {len(obj.users_scene)} scenes")
            bpy.context.collection.objects.unlink(obj)
            bpy.data.objects.remove(obj, do_unlink=True)
            bpy.context.view_layer.update()
        else:
            print("obj was None")
    else:
        print("obj_name was not found")
