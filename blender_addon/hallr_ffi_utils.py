"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import os
import platform
import bpy
import ctypes
from typing import List, Tuple, Dict, Optional
from contextlib import contextmanager
import time
import numpy as np

# workaround for the "ImportError: attempted relative import with no known parent package" problem:
DEV_MODE = False  # Set this to False for distribution
HALLR_LIBRARY = None
LATEST_LOADED_LIBRARY_FILE = None  # Track which file is currently loaded


@contextmanager
def timer(description="Operation"):
    start = time.perf_counter()
    yield
    elapsed = time.perf_counter() - start
    print(f"{description} took {_duration_to_str(elapsed)}")


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
                ("indices", ctypes.POINTER(ctypes.c_uint32)),
                ("indices_count", ctypes.c_size_t),
                ("matrices", ctypes.POINTER(ctypes.c_float)),
                ("matrices_count", ctypes.c_size_t)]


class ProcessResult(ctypes.Structure):
    _fields_ = [("geometry", GeometryOutput),
                ("map", StringMap)]


def _load_latest_dylib(prefix="libhallr_"):
    global HALLR_LIBRARY
    global LATEST_LOADED_LIBRARY_FILE

    if DEV_MODE:
        # this will be find-and-replaced by the build script
        directory = "HALLR__TARGET_RELEASE"

        # List all files in the directory with the given prefix
        files = [f for f in os.listdir(directory) if
                 os.path.isfile(os.path.join(directory, f)) and f.startswith(prefix)]

        # Sort files by their modification time
        files.sort(key=lambda x: os.path.getmtime(os.path.join(directory, x)), reverse=True)

        if not files:
            raise ValueError("Could not find the hallr runtime library!")

        latest_dylib = os.path.join(directory, files[0])

        # Check if we already have this exact file loaded
        if HALLR_LIBRARY is not None and LATEST_LOADED_LIBRARY_FILE == latest_dylib:
            return HALLR_LIBRARY

        # Load the new library
        print("Loading lib: ", latest_dylib)
        rust_lib = ctypes.cdll.LoadLibrary(latest_dylib)
        LATEST_LOADED_LIBRARY_FILE = latest_dylib  # Track what we just loaded

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
                                          ctypes.POINTER(ctypes.c_uint32), ctypes.c_size_t,
                                          ctypes.POINTER(ctypes.c_float), ctypes.c_size_t,
                                          ctypes.POINTER(StringMap)]

    rust_lib.process_geometry.restype = ProcessResult

    rust_lib.free_process_results.argtypes = [ctypes.POINTER(ProcessResult)]
    rust_lib.free_process_results.restype = None
    HALLR_LIBRARY = rust_lib
    return rust_lib


def _ctypes_close_library(lib):
    if DEV_MODE:
        dlclose_func = ctypes.CDLL(None).dlclose
        dlclose_func.argtypes = [ctypes.c_void_p]
        dlclose_func.restype = ctypes.c_int
        dlclose_func(lib._handle)


class MeshFormat:
    """Constants for mesh formats"""
    TRIANGULATED = "△"
    LINE_WINDOWS = "∧"
    EDGES = "⸗"
    POINT_CLOUD = "⁖"


MESH_FORMAT_TAG = "📦"
VERTEX_MERGE_TAG = "≈"
COMMAND_TAG = "▶"

IDENTITY_FFI_MATRIX = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]


def _package_mesh_data(mesh_obj: bpy.types.Object, mesh_format: str = MeshFormat.TRIANGULATED) -> Tuple[
    np.ndarray, np.ndarray]:
    """
    Extract vertices and indices from a Blender mesh object in a consistent format.

    Args:
        mesh_obj: The Blender mesh object to extract data from
        mesh_format: The format to interpret the mesh data

    Returns:
        tuple: (vertices as float32 array, indices as uint32 array)
    """
    mesh = mesh_obj.data
    vertex_count = len(mesh.vertices)

    # Handle vertices - use foreach_get for batch extraction
    world_matrix = mesh_obj.matrix_world

    if world_matrix.is_identity:
        print(f"Python: not applying local-world transformation")
        # Get vertices directly as flat array [x,y,z, x,y,z, ...]
        vertices = np.empty(vertex_count * 3, dtype=np.float32)
        mesh.vertices.foreach_get("co", vertices)
    else:
        print(f"Python: applying local-world transformation.")
        # Get vertices and transform them
        vertices = np.empty(vertex_count * 3, dtype=np.float32)
        mesh.vertices.foreach_get("co", vertices)
        vertices = vertices.reshape(-1, 3)

        # Apply transformation matrix
        # Convert Blender matrix to numpy for vectorized multiplication
        mat = np.array(world_matrix, dtype=np.float32)
        # Transform all vertices at once (homogeneous coordinates)
        ones = np.ones((vertex_count, 1), dtype=np.float32)
        vertices_homo = np.hstack([vertices, ones])
        vertices = (vertices_homo @ mat.T)[:, :3]
        vertices = vertices.flatten()

    # Handle indices based on mesh_format
    if mesh_format == MeshFormat.TRIANGULATED:
        # Verify the mesh is triangulated
        if not all(len(face.vertices) == 3 for face in mesh.polygons):
            raise HallrException(f"The '{mesh_obj.name}' mesh is not fully triangulated!")

        poly_count = len(mesh.polygons)
        if poly_count == 0:
            raise HallrException(f"No polygons found in '{mesh_obj.name}', maybe the mesh is not fully triangulated?")

        # Extract all loop indices at once
        indices = np.empty(poly_count * 3, dtype=np.uint32)
        mesh.loops.foreach_get("vertex_index", indices)

    elif mesh_format == MeshFormat.EDGES:
        # Verify there are no polygons for line formats
        if len(mesh.polygons) > 0:
            raise HallrException(
                f"The '{mesh_obj.name}' model should not contain any polygons for line operations, only edges!")

        edge_count = len(mesh.edges)
        # Extract all edge vertex indices at once
        indices = np.empty(edge_count * 2, dtype=np.uint32)
        mesh.edges.foreach_get("vertices", indices)

    elif mesh_format == MeshFormat.POINT_CLOUD:
        indices = np.empty(0, dtype=np.uint32)
    else:
        raise HallrException(f"The mesh format '{mesh_format}' is not supported!")

    return vertices, indices


def _unpackage_mesh_from_ffi(wall_clock, ffi_vertices_ptr, ffi_indices_ptr, vertex_count, index_count, return_options):
    # print(f"Python: begin unpackage_mesh_from_ffi: {_duration_to_str(time.perf_counter() - wall_clock)}")

    new_mesh = bpy.data.meshes.new(return_options.get("model_0_name", "new_mesh"))
    mesh_format = return_options.get(MESH_FORMAT_TAG, None)

    if mesh_format is None:
        raise HallrException("The mesh format was missing from the return data.")

    # Wrap FFI pointers as NumPy arrays (zero-copy!)
    # FFIVector3 is 3 * f32 = 12 bytes, so we can view it as flat f32 array
    vertex_array = np.ctypeslib.as_array(
        ctypes.cast(ffi_vertices_ptr, ctypes.POINTER(ctypes.c_float)),
        shape=(vertex_count * 3,)
    )
    # the indices are rust::u32
    index_array = np.ctypeslib.as_array(
        ctypes.cast(ffi_indices_ptr, ctypes.POINTER(ctypes.c_uint32)),
        shape=(index_count,)
    )

    new_mesh.vertices.add(vertex_count)
    new_mesh.vertices.foreach_set("co", vertex_array)

    if mesh_format == MeshFormat.TRIANGULATED:
        face_count = index_count // 3
        new_mesh.polygons.add(face_count)
        new_mesh.loops.add(index_count)

        # Batch set everything
        new_mesh.loops.foreach_set("vertex_index", index_array)

        loop_starts = np.arange(0, index_count, 3, dtype=np.int32)
        new_mesh.polygons.foreach_set("loop_start", loop_starts)

        loop_totals = np.full(face_count, 3, dtype=np.int32)
        new_mesh.polygons.foreach_set("loop_total", loop_totals)

    elif mesh_format == MeshFormat.LINE_WINDOWS:
        # Process line mesh in window format (consecutive pairs)
        # Convert [0,1,2,3,4] -> [(0,1), (1,2), (2,3), (3,4)]
        edge_count = index_count - 1
        if edge_count > 0:
            new_mesh.edges.add(edge_count)

            # Create edge pairs: interleave index_array[:-1] and index_array[1:]
            edge_vertices = np.empty(edge_count * 2, dtype=np.int32)
            edge_vertices[0::2] = index_array[:-1]  # Every even index: [0,1,2,3]
            edge_vertices[1::2] = index_array[1:]  # Every odd index:  [1,2,3,4]
            # Result: [0,1, 1,2, 2,3, 3,4]

            new_mesh.edges.foreach_set("vertices", edge_vertices)

    elif mesh_format == MeshFormat.EDGES:
        # Process line mesh in chunks format (paired indices)
        # Already in the right format: [v1,v2, v1,v2, ...]
        edge_count = index_count // 2
        if edge_count > 0:
            new_mesh.edges.add(edge_count)

            # Cast to int32 if needed (Blender expects int32)
            edge_vertices = index_array[:edge_count * 2].astype(np.int32)
            new_mesh.edges.foreach_set("vertices", edge_vertices)

            # Check if the length is odd and print a warning
            if index_count % 2 != 0:
                print("Warning: Length of indices is odd. The last value may not form a valid edge pair.")

    elif mesh_format == MeshFormat.POINT_CLOUD:
        # No additional processing needed - vertices are already set
        pass
    else:
        raise HallrException(f"Mesh format not recognized: {mesh_format}")

    # print(f"Python: done unpackage_mesh_from_ffi: {_duration_to_str(time.perf_counter() - wall_clock)}")

    # Update the mesh to ensure proper calculation of derived data
    mesh_update_start = time.perf_counter()
    new_mesh.update(calc_edges=True)
    # print(f"Python: done mesh_update: {_duration_to_str(time.perf_counter() - wall_clock)}")

    return new_mesh


def _handle_new_object(return_options: Dict[str, str],
                       mesh_obj: bpy.types.Object,
                       select_new_mesh: bool = True) -> None:
    """
    Set up new object properties. Must be called in OBJECT mode.

    Args:
        return_options: Dictionary of options for post-processing
        mesh_obj: The new mesh object to handle
        select_new_mesh: Whether to select the new mesh
    """
    # Assert we're in OBJECT mode
    assert bpy.context.object is None or bpy.context.object.mode == 'OBJECT', \
        "handle_new_object must be called in OBJECT mode"

    # Set flat shading directly on mesh
    for poly in mesh_obj.data.polygons:
        poly.use_smooth = False

    if select_new_mesh:
        # Deselect all objects
        bpy.ops.object.select_all(action='DESELECT')

        # Select and make active
        mesh_obj.select_set(True)
        bpy.context.view_layer.objects.active = mesh_obj

    # Handle vertex merging
    if VERTEX_MERGE_TAG in return_options:
        try:
            remove_doubles_threshold = float(return_options.get(VERTEX_MERGE_TAG))
            print(f"Python: removing doubles by {remove_doubles_threshold}")
            with timer("Python: bmesh.ops.remove_doubles()"):
                _merge_vertices_bmesh(mesh_obj.data, remove_doubles_threshold)
        except ValueError:
            pass


def _update_existing_object(wall_clock, active_obj: bpy.types.Object,
                            return_options: Dict[str, str],
                            new_mesh) -> None:
    """
    Update object mesh data. Must be called in OBJECT mode.
    """

    # Assert we're in OBJECT mode
    assert bpy.context.object is None or bpy.context.object.mode == 'OBJECT', \
        "update_existing_object must be called in OBJECT mode"

    # Store reference to old mesh
    old_mesh = active_obj.data

    # Replace mesh data
    active_obj.data = new_mesh

    # Force complete update
    new_mesh.update()
    new_mesh.update_tag()
    active_obj.update_tag(refresh={'OBJECT', 'DATA'})

    # Set flat shading directly on mesh polygons
    for poly in new_mesh.polygons:
        poly.use_smooth = False

    # Clean up old mesh
    if old_mesh.users == 0 and not old_mesh.use_fake_user:
        bpy.data.meshes.remove(old_mesh)

    # Handle vertex merging using BMesh (no mode change needed)
    if VERTEX_MERGE_TAG in return_options:
        try:
            remove_doubles_threshold = float(return_options.get(VERTEX_MERGE_TAG))
            print(f"Python: removing doubles by {remove_doubles_threshold} (bmesh)")
            with timer("Python: bmesh.ops.remove_doubles()"):
                _merge_vertices_bmesh(new_mesh, remove_doubles_threshold)
        except ValueError as e:
            print(f"ValueError details: {str(e)}")

    #print(f"Python: update_existing_object: {_duration_to_str(time.perf_counter() - wall_clock)}")


def apply_all_transformations(obj: bpy.types.Object, temp_name: str) -> Tuple[bpy.types.Object, bool]:
    """
    Prepare an object for processing by handling transforms.
    When applying transforms it's easier to do so on a copy of the original object.

    Args:
        obj: The object to prepare
        temp_name: Name to use for the duplicated object if needed

    Returns:
        tuple: (prepared_object, is_duplicated) - the object to use and whether it's a duplicate
    """
    # Deselect all objects
    bpy.ops.object.select_all(action='DESELECT')

    # Select only the object we want to duplicate
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj

    if _has_un_applied_transformations(obj):
        # Duplicate the object
        bpy.ops.object.duplicate()
        dup_obj = bpy.context.object
        dup_obj.name = temp_name

        # Apply all transformations
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
        return dup_obj, True

    return obj, False


def apply_all_transformations_direct(obj: bpy.types.Object) -> bpy.types.Object:
    """
    Prepare an object for processing by applying transformations directly.

    Args:
        obj: The object to prepare

    Returns:
        The prepared object
    """
    if _has_un_applied_transformations(obj):
        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

    return obj


def process_mesh_with_rust(wall_clock,
                           config: Dict[str, str],
                           primary_object: Optional[bpy.types.Object] = None,
                           secondary_object: Optional[bpy.types.Object] = None,
                           primary_format: Optional[str] = None,
                           secondary_format: Optional[str] = None,
                           create_new: bool = True) -> Optional[bpy.types.Object]:
    """
    Process mesh data with the Rust library.
    This is the ONLY function that handles mode switching.
    All lower-level functions assume OBJECT mode.

    Args:
        wall_clock: the start time of the operation execution
        config: Dictionary of configuration options for Rust
        primary_object: Optional Primary Blender mesh object (can be None)
        secondary_object: Optional secondary mesh object
        primary_format: Optional Format for the primary mesh
        secondary_format: Optional Format for the secondary mesh
        create_new: If True, create a new object; if False, modify the active object

    Returns:
        If create_new is True: The newly created object, a status message (string)
        If create_new is False: None (the active object is modified), a status message (string)
    """
    # Store the original mode at the very start
    original_mode = bpy.context.object.mode if bpy.context.object else 'OBJECT'

    try:
        # Ensure OBJECT mode for all operations
        if bpy.context.object and bpy.context.object.mode != 'OBJECT':
            bpy.ops.object.mode_set(mode='OBJECT')

        result = _process_mesh_with_rust(wall_clock, config, primary_object, secondary_object, primary_format,
                                         secondary_format,
                                         create_new)
        return result[0], result[1] + f" duration:{_duration_to_str(time.perf_counter() - wall_clock)}"

    finally:
        # CRITICAL: Force dependency graph update before returning to edit mode
        bpy.context.view_layer.update()

        # Restore original mode
        if original_mode != 'OBJECT':
            bpy.ops.object.mode_set(mode=original_mode)

        # print(f"Python: context_view_layer_update: {_duration_to_str(time.perf_counter() - wall_clock)}")


def _process_mesh_with_rust(wall_clock,
                            config: Dict[str, str],
                            primary_object: Optional[bpy.types.Object] = None,
                            secondary_object: Optional[bpy.types.Object] = None,
                            primary_format: Optional[str] = None,
                            secondary_format: Optional[str] = None,
                            create_new: bool = True) -> Optional[bpy.types.Object]:
    """
    Process mesh data with the Rust library.

    Args:
        config: Dictionary of configuration options for Rust
        primary_object: Optional Primary Blender mesh object (can be None)
        secondary_object: Optional secondary mesh object
        primary_format: Optional Format for the primary mesh
        secondary_format: Optional Format for the secondary mesh
        create_new: If True, create a new object; if False, modify the active object

    Returns:
        If create_new is True: The newly created object, a status message (string)
        If create_new is False: None (the active object is modified), a status message (string)
    """

    # print(f"Python: begin marshal: {_duration_to_str(time.perf_counter() - wall_clock)}")

    original_vertices = 0
    original_indices = 0

    if create_new:
        new_object = bpy.data.objects.new("New_Object", bpy.data.meshes.new("empty mesh"))
        bpy.context.collection.objects.link(new_object)

        # Create a custom undo step - this will just delete the new object when undone
        # This prevents blender from crashing on ctrl-Z, but it also, occasionally, leaves the empty "New_Object" behind.
        bpy.ops.ed.undo_push(message="Create New Object")

        new_object.select_set(True)
        bpy.context.view_layer.objects.active = new_object

    # Set up mesh formats in config
    mesh_format = ""
    if primary_object:
        mesh_format += primary_format

    # Prepare data structures
    # Prepare data structures - start with empty arrays
    vertices = np.empty(0, dtype=np.float32)
    indices = np.empty(0, dtype=np.uint32)
    matrices = np.empty(0, dtype=np.float32)

    # Store the current selection and active object state
    # selected_objects = [o for o in bpy.context.selected_objects]

    input_marshal_start = time.perf_counter()

    if primary_object:
        primary_obj_to_process = primary_object
        primary_vertices, primary_indices = _package_mesh_data(primary_obj_to_process, primary_format)
        original_vertices += len(primary_vertices) // 3
        original_indices += len(primary_indices)

        vertices = primary_vertices
        indices = primary_indices
        matrices = _get_matrices_col_major(primary_object)  # Returns ndarray now

    if secondary_object:
        mesh_format += secondary_format
        first_vertex_model_1 = len(vertices) // 3
        first_index_model_1 = len(indices)
        config["first_vertex_model_1"] = str(first_vertex_model_1)
        config["first_index_model_1"] = str(first_index_model_1)

        secondary_vertices, secondary_indices = _package_mesh_data(secondary_object, secondary_format)
        original_vertices += len(secondary_vertices) // 3
        original_indices += len(secondary_indices)

        vertices = np.concatenate([vertices, secondary_vertices])
        indices = np.concatenate([indices, secondary_indices])
        matrices = np.concatenate([matrices, _get_matrices_col_major(secondary_object)])  # ← Concatenate!

    if mesh_format != "":
        config[MESH_FORMAT_TAG] = mesh_format

    # print(f"Python: done input marshal: {_duration_to_str(time.perf_counter() - wall_clock)}")

    # Cast flat float array to Vector3 pointer
    vertices_ptr = vertices.ctypes.data_as(ctypes.POINTER(Vector3))

    # Convert to ctypes pointers - all ndarrays now!
    indices_ptr = indices.ctypes.data_as(ctypes.POINTER(ctypes.c_uint32))
    matrices_ptr = matrices.ctypes.data_as(ctypes.POINTER(ctypes.c_float))

    # Create StringMap from config
    keys_list = list(config.keys())
    values_list = list(config.values())
    keys_array = (ctypes.c_char_p * len(keys_list))(*[k.encode('utf-8') for k in keys_list])
    values_array = (ctypes.c_char_p * len(values_list))(*[v.encode('utf-8') for v in values_list])
    map_data = StringMap(keys_array, values_array, len(keys_list))

    print(f"Python: {COMMAND_TAG} '{config.get(COMMAND_TAG, '')}'")
    print(f"Python: sending {len(vertices) // 3} vertices, {len(indices)} indices, {len(matrices) // 16} matrices")

    # print(f"Python: updating dylib : {_duration_to_str(time.perf_counter() - wall_clock)}")
    # Fetch Rust library
    rust_lib = _load_latest_dylib()

    # print(f"Python: start actual_rust_call: {_duration_to_str(time.perf_counter() - wall_clock)}")

    # Call Rust function
    rust_result = rust_lib.process_geometry(
        vertices_ptr, len(vertices) // 3, indices_ptr, len(indices),
        matrices_ptr, len(matrices), map_data
    )
    # print(f"Python: done actual_rust_call: {_duration_to_str(time.perf_counter() - wall_clock)}")
    try:
        # Extract return options
        return_options = {}
        for i in range(rust_result.map.count):
            key = ctypes.string_at(rust_result.map.keys[i]).decode('utf-8')
            value = ctypes.string_at(rust_result.map.values[i]).decode('utf-8')
            if key == "ERROR":
                raise HallrException(value)
            return_options[key] = value

        new_mesh = _unpackage_mesh_from_ffi(wall_clock, rust_result.geometry.vertices, rust_result.geometry.indices,
                                            rust_result.geometry.vertex_count, rust_result.geometry.indices_count,
                                            return_options)
        indices_count = rust_result.geometry.indices_count
    finally:
        # Free Rust memory
        rust_lib.free_process_results(rust_result)
    # print(f"Python: after free results: {_duration_to_str(time.perf_counter() - wall_clock)}")

    if DEV_MODE:
        _ctypes_close_library(rust_lib)

    print("Python: received config: ", return_options)
    print(
        f"Python: received {len(new_mesh.vertices)} vertices, {indices_count} indices, {len(new_mesh.edges)} edges, {len(new_mesh.polygons)} polygons")

    # Create or update object based on results
    if create_new:
        print("Python: new object new mesh")
        # Create a new object

        new_object.data = new_mesh
        # Handle the new object (link to scene, select, etc.)
        _handle_new_object(return_options, new_object)
        return new_object, f"New mesh: vertices:{len(new_mesh.vertices)} indices:{indices_count}"
    else:
        print("Python: updating old object with new mesh")
        # Update existing object
        bpy.context.view_layer.objects.active = primary_object
        _update_existing_object(wall_clock, primary_object, return_options, new_mesh)
        return None, f"Modified mesh: Δvertices:{len(new_mesh.vertices) - original_vertices} Δindices:{indices_count - original_indices}"


# Simpler convenience functions that wrap the main processing function

def process_single_mesh(wall_clock,
                        config: Dict[str, str], mesh_obj: bpy.types.Object = None,
                        mesh_format: str = MeshFormat.EDGES,
                        create_new: bool = True) -> Optional[bpy.types.Object]:
    """
    Process a single mesh with the Rust library.

    Args:
        config: Configuration options
        mesh_obj: The mesh object to process
        mesh_format: Mesh format (windows or chunks)
        create_new: Whether to create a new object or modify the active one

    Returns:
        (The new object, info message) if create_new is True, (None, info message) otherwise
    """
    return process_mesh_with_rust(
        wall_clock,
        config,
        primary_object=mesh_obj,
        primary_format=mesh_format,
        create_new=create_new
    )


def process_config(wall_clock, config: Dict[str, str]) -> bpy.types.Object:
    """
    Process a command that does not require an input object, only a config.

    Args:
        config: Configuration options

    Returns:
        The new object, info message
    """
    return process_mesh_with_rust(
        wall_clock,
        config,
        primary_object=None,
        secondary_object=None,
        create_new=True
    )


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


def _get_matrices_col_major(bpy_object):
    """ Return the world orientation as an array of 16 floats in column-major order"""
    bm = bpy_object.matrix_world
    # Convert to numpy array (4x4) and flatten in column-major ('F'ortran) order
    return np.array(bm, dtype=np.float32).flatten(order='F')


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


def _has_un_applied_transformations(obj):
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


def _merge_vertices_bmesh(mesh: bpy.types.Mesh, threshold: float) -> None:
    """Merge vertices using BMesh without changing modes."""
    bm = bmesh.new()
    bm.from_mesh(mesh)

    # Remove doubles
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=threshold)

    # Write back to mesh
    bm.to_mesh(mesh)
    bm.free()

    # Update mesh to reflect changes
    mesh.update()


def _duration_to_str(duration):
    units = [
        ('s', 1),
        ('ms', 1e3),
        ('μs', 1e6),
        ('ns', 1e9)
    ]

    for unit, factor in units:
        if duration * factor >= 1 or unit == 'ns':
            value = duration * factor
            return f"{value:.2f} {unit}"
    # this fallback is actually never used
    return f"{duration:.2f} s"
