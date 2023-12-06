"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
import os
import platform
from importlib import reload
import ctypes
import bmesh
import mathutils

# workaround for the "ImportError: attempted relative import with no known parent package" problem:
DEV_MODE = False  # Set this to False for distribution
HALLR_LIBRARY = None


class HallrException(Exception):
    def __init__(self, message):
        self.message = str(message)


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
                ("indices_count", ctypes.c_size_t),
                ("matrices", ctypes.POINTER(ctypes.c_float)),
                ("matrices_count", ctypes.c_size_t)]


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
                                          ctypes.POINTER(ctypes.c_float), ctypes.c_size_t,
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


# TODO: unify all "new" object handlers
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


# TODO: unify all "new" object handlers
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
    print("inside handle_windows_line_modify_active_object")
    # Ensure that the active object is a mesh object
    active_obj = bpy.context.view_layer.objects.active
    if not active_obj or active_obj.type != 'MESH':
        print("No active mesh object to modify!")
        return

    # Convert the indices to Blender's edge format
    edges = [(indices[i], indices[i + 1]) for i in range(len(indices) - 1)]

    # Free the existing geometry
    bpy.ops.object.mode_set(mode='EDIT')  # Must be in edit mode to use bmesh
    active_obj.data.update()
    if hasattr(active_obj.data, 'bmesh'):
        active_obj.data.bmesh.free()
    # Create a new BMesh
    bm = bmesh.new()

    # Create new vertices and edges
    verts = [bm.verts.new(vert) for vert in vertices]
    bm.from_pydata(verts, edges, [])
    bmesh.update_edit_mesh(active_obj.data)  # Update the mesh with the changes

    bpy.ops.object.mode_set(mode='OBJECT')  # Switch back to object mode

    bm.to_mesh(active_obj.data)
    bm.free()

    # Update the mesh
    active_obj.data.update()


def unpack_model(options, raw_indices):
    """Convert the received data into blender mesh edges, faces and world transform"""
    rv_edges = []
    rv_faces = []
    mesh_format = options.get("mesh.format", None)
    if mesh_format == "line_windows":
        # Convert the indices to Blender's edge format
        # This mode assumes that the line is in the ".window(2)" format,
        # i.e., indices are [0, 1, 2, 3, ...], where [(0,1),(1,2),...] forms edges.
        rv_edges = [(raw_indices[i], raw_indices[i + 1]) for i in range(len(raw_indices) - 1)]
    elif mesh_format == "line_chunks":
        # This mode assumes that the line is in the ".chunks(2)" format,
        # i.e., indices are [0, 1, 2, 3, ...], where [(0,1), (2,3),...] forms edges.
        rv_edges = [(raw_indices[i], raw_indices[i + 1]) for i in range(0, len(raw_indices) - 1, 2)]
    elif mesh_format == "triangulated":
        # Assuming indices are [0, 1, 2, 2, 3, 4, ...], where each set of 3 is a triangle
        rv_faces = [tuple(raw_indices[i:i + 3]) for i in range(0, len(raw_indices), 3)]
    else:
        raise HallrException("Unsupported mesh_format:" + mesh_format)

    # if pb_model.HasField("worldOrientation"):
    #    pbm = pb_model.worldOrientation
    #    mat[0][0], mat[0][1], mat[0][2], mat[0][3] = pbm.m00, pbm.m01, pbm.m02, pbm.m03
    #    mat[1][0], mat[1][1], mat[1][2], mat[1][3] = pbm.m10, pbm.m11, pbm.m12, pbm.m13
    #    mat[2][0], mat[2][1], mat[2][2], mat[2][3] = pbm.m20, pbm.m21, pbm.m22, pbm.m23
    #    mat[3][0], mat[3][1], mat[3][2], mat[3][3] = pbm.m30, pbm.m31, pbm.m32, pbm.m33
    return rv_edges, rv_faces, mathutils.Matrix.Identity(4)


def handle_received_object_replace_active(active_object, options, ffi_vertices, ffi_indices):
    """Takes care of the raw ffi data received from rust, and create a blender mesh out of them"""

    remove_doubles = False
    remove_doubles_threshold = 0.0001

    for key, value in options.items():
        if key == "ERROR":
            raise HallrException(str(value))
        if key == "REMOVE_DOUBLES" and value.lower() == "true":
            remove_doubles = True
        if key == "REMOVE_DOUBLES_THRESHOLD":
            try:
                new_value = float(value)
                remove_doubles_threshold = new_value
            except ValueError:
                pass

    if len(ffi_vertices) == 0 or len(ffi_indices) == 0:
        raise HallrException("No return models found")

    (edges, faces, matrix) = unpack_model(options, ffi_indices)
    if len(faces) > 0 or len(edges) > 0:
        new_mesh = bpy.data.meshes.new(options.get("model_0_name", "new_mesh"))
        old_mesh = active_object.data

        print("vertices:", len(ffi_vertices))
        print("edges:", len(edges))
        print("faces:", len(faces))
        new_mesh.from_pydata(ffi_vertices, edges, faces)
        new_mesh.update(calc_edges=True)
        bm = bmesh.new()
        bm.from_mesh(new_mesh)
        bpy.ops.object.mode_set(mode='OBJECT')
        bm.to_mesh(active_object.data)
        bpy.ops.object.mode_set(mode='EDIT')

        # print("active_object.update_from_editmode():", active_object.update_from_editmode())
        if not (old_mesh.users or old_mesh.use_fake_user):
            bpy.data.meshes.remove(old_mesh)
            print("removed old mesh")
        else:
            print("did not remove old mesh")

        if matrix:
            active_object.matrix_world = matrix

        if remove_doubles:
            # sometimes 'mode_set' does not take right away  :/
            # bpy.ops.object.editmode_toggle()
            bpy.ops.object.mode_set(mode='EDIT')
            bpy.ops.mesh.remove_doubles(threshold=remove_doubles_threshold)
            bpy.ops.object.editmode_toggle()
            bpy.ops.object.mode_set(mode='OBJECT')
            bpy.ops.object.mode_set(mode='EDIT')
        # if set_origin_to_cursor:
        #    bpy.ops.object.origin_set(type='ORIGIN_CURSOR')
    else:
        print("handle_received_object() error: len(faces):", len(faces), " len(edges):", len(edges))


def handle_chunks_line_modify_active_object(vertices, indices):
    """
    Convert vertices and indices to a bpy data, and insert into active object.
    This function assumes that the line is in the ".chunks(2)" format,
    i.e., indices are [0, 1, 2, 3, ...], where [(1,2), (3,4),...] forms edges.
    """
    print("inside handle_chunks_line_modify_active_object")
    # Ensure that the active object is a mesh object
    active_obj = bpy.context.view_layer.objects.active
    if not active_obj or active_obj.type != 'MESH':
        print("No active mesh object to modify!")
        return

    # Convert the indices to Blender's edge format
    # print(indices)
    edges = [(indices[i], indices[i + 1]) for i in range(0, len(indices) - 1, 2)]
    # print(edges)

    # Check if the length is odd and print a warning
    if len(indices) % 2 != 0:
        print("Warning: Length of indices is odd. The last value may not form a valid edge pair.")
        print("indices:", indices)
        print("edges:", edges)

    # Free the existing geometry
    bpy.ops.object.mode_set(mode='EDIT')  # Must be in edit mode to use bmesh
    active_obj.data.update()
    if hasattr(active_obj.data, 'bmesh'):
        active_obj.data.bmesh.free()
    # Create a new BMesh
    bm = bmesh.new()

    # Create new vertices and edges
    verts = [bm.verts.new(vert) for vert in vertices]
    bm.from_pydata(verts, edges, [])
    # Free the existing BMesh data (if any)
    if active_obj.data.is_editmode:
        # If in edit mode, toggle back to object mode
        bpy.ops.object.mode_set(mode='OBJECT')

    bm.to_mesh(active_obj.data)
    bm.free()

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

    if bounding_shape:

        first_vertex_model_1 = len(vertices)
        first_index_model_1 = len(indices)
        # Appending vertices from the bounding shape
        vertices += [Vector3(v.co.x, v.co.y, v.co.z) for v in bounding_obj_to_process.data.vertices]

        config["first_vertex_model_1"] = str(first_vertex_model_1)
        config["first_index_model_1"] = str(first_index_model_1)

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

    # Handle the world orientation
    matrices = get_matrices(active_obj)
    if bounding_shape:
        matrices.extend(get_matrices(bounding_shape))

    matrices_ptr = (ctypes.c_float * len(matrices))(*matrices)

    # 7. Convert the dictionary to two separate lists for keys and values
    keys_list = list(config.keys())
    values_list = list(config.values())
    keys_array = (ctypes.c_char_p * len(keys_list))(*[k.encode('utf-8') for k in keys_list])
    values_array = (ctypes.c_char_p * len(values_list))(*[v.encode('utf-8') for v in values_list])
    map_data = StringMap(keys_array, values_array, len(keys_list))

    # 8. Make the call to rust
    rust_result = rust_lib.process_geometry(vertices_ptr, len(vertices), indices_ptr, len(indices), matrices_ptr,
                                            len(matrices), map_data)

    print("python received: ", rust_result.geometry.vertex_count, "vertices",
          rust_result.geometry.indices_count, "indices")
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
    # When running in release mode, this does nothing.
    ctypes_close_library(rust_lib)

    return output_vertices, output_indices, output_map


def get_matrices(bpy_object):
    """ Return the world orientation as an array of 16 floats"""
    bm = bpy_object.matrix_world
    return [bm[0][0], bm[0][1], bm[0][2], bm[0][3],
            bm[1][0], bm[1][1], bm[1][2], bm[1][3],
            bm[2][0], bm[2][1], bm[2][2], bm[2][3],
            bm[3][0], bm[3][1], bm[3][2], bm[3][3]]


def call_rust_direct(config, active_obj, use_line_chunks=False):
    """
    A simpler version of call_rust that only processes the active_object.
    When `expect_line_chunks` is set, the data will iterate over each edge(a,b) and use a list of
    indices in .chunks(2) format.
    If `expect_line_chunks` is not set, the code expect the mesh to be triangulated.
    """

    rust_lib = load_latest_dylib()

    active_obj_to_process = prepare_object_for_processing_direct(active_obj)
    # handle the vertices
    vertices = [Vector3(v.co.x, v.co.y, v.co.z) for v in active_obj_to_process.data.vertices]
    vertices_ptr = (Vector3 * len(vertices))(*vertices)

    # Handle the indices
    if use_line_chunks:
        config["mesh.format"] = "line_chunks"
        if len(active_obj_to_process.data.polygons) > 0:
            raise HallrException("The model should not contain any polygons for this operation, only edges! Hint: use "
                                 "the 2d_outline operation to convert a mesh to a 2d outline.")
        indices = [v for edge in active_obj_to_process.data.edges for v in edge.vertices]
    else:
        config["mesh.format"] = "triangulated"
        # Collect vertices and check if the mesh is fully triangulated
        indices = []
        for face in active_obj_to_process.data.polygons:
            if len(face.vertices) != 3:
                raise HallrException("The mesh is not fully triangulated!")
            indices.extend(face.vertices)
        if len(indices) == 0:
            raise HallrException("No polygons found, maybe the mesh is not fully triangulated?")
    indices_ptr = (ctypes.c_size_t * len(indices))(*indices)

    # Handle the world orientation
    matrices = get_matrices(active_obj)
    matrices_ptr = (ctypes.c_float * len(matrices))(*matrices)

    # Handle the StringMap
    keys_list = list(config.keys())
    values_list = list(config.values())
    keys_array = (ctypes.c_char_p * len(keys_list))(*[k.encode('utf-8') for k in keys_list])
    values_array = (ctypes.c_char_p * len(values_list))(*[v.encode('utf-8') for v in values_list])
    map_data = StringMap(keys_array, values_array, len(keys_list))

    # This calls the rust library
    rust_result = rust_lib.process_geometry(vertices_ptr, len(vertices), indices_ptr, len(indices), matrices_ptr,
                                            len(matrices), map_data)

    output_vertices = [(vec.x, vec.y, vec.z) for vec in
                       (rust_result.geometry.vertices[i] for i in range(rust_result.geometry.vertex_count))]
    output_indices = [rust_result.geometry.indices[i] for i in range(rust_result.geometry.indices_count)]

    output_map = {}
    for i in range(rust_result.map.count):
        key = ctypes.string_at(rust_result.map.keys[i]).decode('utf-8')
        value = ctypes.string_at(rust_result.map.values[i]).decode('utf-8')
        output_map[key] = value
    # This should free the data owned by Rust
    rust_lib.free_process_results(rust_result)
    # In development mode this tries to close the library, in release mode it does nothing
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
