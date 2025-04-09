"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import os
import platform
import bpy
import ctypes
import bmesh
from typing import List, Tuple, Dict, Optional

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


class MeshFormat:
    """Constants for mesh formats"""
    TRIANGULATED = "△"
    LINE_WINDOWS = "∧"
    LINE_CHUNKS = "⸗"
    POINT_CLOUD = "┅"

MESH_FORMAT_TAG = "📦"

def package_mesh_data(mesh_obj: bpy.types.Object, mesh_format: str = MeshFormat.TRIANGULATED,
                      only_selected_vertices: bool = False) -> Tuple[List, List]:
    """
    Extract vertices and indices from a Blender mesh object in a consistent format.

    Args:
        mesh_obj: The Blender mesh object to extract data from
        mesh_format: The format to interpret the mesh data
        only_selected_vertices: If True, only include selected vertices

    Returns:
        tuple: (vertices, indices) in the format specified
    """
    # Handle vertices
    if only_selected_vertices:
        vertices = [Vector3(v.co.x, v.co.y, v.co.z) for v in mesh_obj.data.vertices if v.select]
    else:
        vertices = [Vector3(v.co.x, v.co.y, v.co.z) for v in mesh_obj.data.vertices]

    # Handle indices based on mesh_format
    indices = []
    if mesh_format == MeshFormat.TRIANGULATED:
        # Verify the mesh is triangulated
        if not all(len(face.vertices) == 3 for face in mesh_obj.data.polygons):
            raise HallrException(f"The '{mesh_obj.name}' mesh is not fully triangulated!")

        indices = [v for face in mesh_obj.data.polygons for v in face.vertices]
        if len(indices) == 0:
            raise HallrException(f"No polygons found in '{mesh_obj.name}', maybe the mesh is not fully triangulated?")

    elif mesh_format in [MeshFormat.LINE_WINDOWS, MeshFormat.LINE_CHUNKS]:
        # Verify there are no polygons for line formats
        if len(mesh_obj.data.polygons) > 0:
            raise HallrException(
                f"The '{mesh_obj.name}' model should not contain any polygons for line operations, only edges!")

        # Get edges in appropriate format
        indices = [v for edge in mesh_obj.data.edges for v in edge.vertices]
    elif mesh_format != MeshFormat.POINT_CLOUD:
        raise HallrException(f"The mesh format '{mesh_format}' is not supported!")

    return vertices, indices


def handle_new_object(return_options: Dict[str, str], mesh_obj: bpy.types.Object, select_new_mesh: bool = True) -> None:
    """
    Link a new mesh object to the scene and set up its properties.

    Args:
        return_options: Dictionary of options for post-processing
        mesh_obj: The new mesh object to handle
        select_new_mesh: Whether to select the new mesh
    """
    bpy.context.collection.objects.link(mesh_obj)

    if select_new_mesh:
        # Make the new object active
        bpy.context.view_layer.objects.active = mesh_obj

        # Ensure object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        # Deselect all objects
        bpy.ops.object.select_all(action='DESELECT')

        # Select the newly created object
        mesh_obj.select_set(True)

    # Process post-creation options
    remove_doubles = False
    remove_doubles_threshold = 0.00001

    for key, value in return_options.items():
        if key == "ERROR":
            raise HallrException(str(value))
        if key == "REMOVE_DOUBLES" and value.lower() == "true":
            remove_doubles = True
        if key == "REMOVE_DOUBLES_THRESHOLD":
            try:
                remove_doubles_threshold = float(value)
            except ValueError:
                pass

    if remove_doubles:
        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.remove_doubles(threshold=remove_doubles_threshold)
        bpy.ops.object.editmode_toggle()
        bpy.ops.object.mode_set(mode='OBJECT')
        bpy.ops.object.mode_set(mode='EDIT')


def create_mesh_object(vertices: List[Tuple[float, float, float]],
                       edges: List[Tuple[int, int]] = None,
                       faces: List[Tuple[int, ...]] = None,
                       name: str = "New_Object") -> bpy.types.Object:
    """
    Create a new mesh object from vertices, edges, and faces.

    Args:
        vertices: List of vertex coordinates
        edges: List of edge vertex pairs
        faces: List of face vertex indices
        name: Name for the new object

    Returns:
        The newly created object
    """
    if edges is None:
        edges = []
    if faces is None:
        faces = []

    # Create a new mesh
    mesh = bpy.data.meshes.new(name=f"{name}_Mesh")
    mesh.from_pydata(vertices, edges, faces)
    mesh.update()

    # Create a new object using the mesh
    mesh_obj = bpy.data.objects.new(name, mesh)

    return mesh_obj


def unpack_mesh_data(indices: List[int], mesh_format: str) -> Tuple[List[Tuple[int, int]], List[Tuple[int, ...]]]:
    """
    Convert indices to edges and faces based on the specified format.

    Args:
        indices: List of indices
        mesh_format: The format of the indices

    Returns:
        tuple: (edges, faces) - lists of edge and face vertex indices
    """
    edges = []
    faces = []

    if mesh_format == MeshFormat.TRIANGULATED:
        print("unpacking triangles")
        # Process triangulated mesh
        faces = [tuple(indices[i:i + 3]) for i in range(0, len(indices), 3)]
    elif mesh_format == MeshFormat.LINE_WINDOWS:
        print("unpacking line windows")
        # Process line mesh in window format (consecutive pairs)
        edges = [(indices[i], indices[i + 1]) for i in range(len(indices) - 1)]
    elif mesh_format == MeshFormat.LINE_CHUNKS:
        print("unpacking line chunks")
        # Process line mesh in chunks format (paired indices)
        edges = [(indices[i], indices[i + 1]) for i in range(0, len(indices) - 1, 2)]

        # Check if the length is odd and print a warning
        if len(indices) % 2 != 0:
            print("Warning: Length of indices is odd. The last value may not form a valid edge pair.")
    else:
        raise HallrException(f"Mesh format not recognized:{mesh_format}")

    return edges, faces


def create_object_from_mesh_data(return_options: Dict[str, str],
                                 vertices: List[Tuple[float, float, float]],
                                 indices: List[int],
                                 mesh_format: str,
                                 name: str = "New_Object") -> bpy.types.Object:
    """
    Create a new Blender object from vertices and indices.

    Args:
        return_options: Dictionary of options for post-processing
        vertices: List of vertex coordinates
        indices: List of indices
        mesh_format: The format of the indices
        name: Name for the new object

    Returns:
        The newly created object
    """
    edges, faces = unpack_mesh_data(indices, mesh_format)

    # Create mesh object
    mesh_obj = create_mesh_object(vertices, edges, faces, name)

    # Handle the new object (link to scene, select, etc.)
    handle_new_object(return_options, mesh_obj)

    return mesh_obj


def update_existing_object(active_obj: bpy.types.Object,
                           return_options: Dict[str, str],
                           vertices: List[Tuple[float, float, float]],
                           indices: List[int],
                           mesh_format: str) -> None:
    """
    Update an existing Blender object with new vertices and indices.

    Args:
        active_obj: The Blender object to update
        return_options: Dictionary of options for post-processing
        vertices: List of vertex coordinates
        indices: List of indices
        mesh_format: The format of the indices
    """
    # Convert indices to edges and faces
    edges, faces = unpack_mesh_data(indices, mesh_format)

    # Create new mesh
    new_mesh = bpy.data.meshes.new(return_options.get("model_0_name", "new_mesh"))
    old_mesh = active_obj.data

    # Update the mesh
    new_mesh.from_pydata(vertices, edges, faces)
    new_mesh.update(calc_edges=True)

    # Use bmesh to update the active object
    bm = bmesh.new()
    bm.from_mesh(new_mesh)

    # Switch to object mode for mesh operations
    bpy.ops.object.mode_set(mode='OBJECT')
    bm.to_mesh(active_obj.data)
    bpy.ops.object.mode_set(mode='EDIT')

    # Clean up old mesh if not needed
    if not (old_mesh.users or old_mesh.use_fake_user):
        print("would have removed old mesh")
        #bpy.data.meshes.remove(old_mesh)

    # Apply matrix if provided in return options
    if "matrix" in return_options:
        try:
            # Note: This is just a placeholder - actual implementation would depend on
            # how the matrix is encoded in return_options
            # active_obj.matrix_world = matrix_from_return_options(return_options["matrix"])
            pass
        except Exception as e:
            print(f"Error applying matrix: {e}")

    # Handle remove doubles option
    remove_doubles = return_options.get("REMOVE_DOUBLES", "").lower() == "true"
    remove_doubles_threshold = float(return_options.get("REMOVE_DOUBLES_THRESHOLD", "0.00001"))

    if remove_doubles:
        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.remove_doubles(threshold=remove_doubles_threshold)
        bpy.ops.object.editmode_toggle()
        bpy.ops.object.mode_set(mode='OBJECT')
        bpy.ops.object.mode_set(mode='EDIT')


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

    if has_un_applied_transformations(obj):
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
    if has_un_applied_transformations(obj):
        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

    return obj


def process_mesh_with_rust(config: Dict[str, str],
                           primary_mesh: Optional[bpy.types.Object] = None,
                           secondary_mesh: Optional[bpy.types.Object] = None,
                           primary_format: Optional[str] = None,
                           secondary_format: Optional[str] = None,
                           create_new: bool = True,
                           only_selected_vertices: bool = False) -> Optional[bpy.types.Object]:
    """
    Process mesh data with the Rust library.

    Args:
        config: Dictionary of configuration options for Rust
        primary_mesh: Primary Blender mesh object (can be None)
        secondary_mesh: Optional secondary mesh object
        primary_format: Optional Format for the primary mesh
        secondary_format: Optional Format for the secondary mesh
        create_new: If True, create a new object; if False, modify the active object
        only_selected_vertices: If True, only process selected vertices in the primary mesh

    Returns:
        If create_new is True: The newly created object
        If create_new is False: None (the active object is modified)
    """
    # Set up mesh formats in config
    mesh_format = ""
    if primary_mesh:
        mesh_format += primary_format

    # Prepare data structures
    vertices = []
    indices = []
    matrices = []

    # Process primary mesh if provided
    primary_obj_to_process = None
    primary_obj_is_duplicated = False

    if primary_mesh:
        # Prepare object with transformations
        primary_obj_to_process, primary_obj_is_duplicated = apply_all_transformations(
            primary_mesh, "TempDuplicateActive"
        )

        # Extract mesh data
        if only_selected_vertices:
            vertices = [Vector3(v.co.x, v.co.y, v.co.z) for v in primary_mesh.data.vertices if v.select]
            indices = []
        else:
            primary_vertices, primary_indices = package_mesh_data(primary_obj_to_process, primary_format)
            vertices.extend(primary_vertices)
            indices.extend(primary_indices)

        # Get transformation matrices
        matrices.extend(get_matrices(primary_mesh))
    else:
        # Use identity matrix if no primary mesh
        matrices = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]

    # Process secondary mesh if provided
    secondary_obj_to_process = None
    secondary_obj_is_duplicated = False

    if secondary_mesh:
        mesh_format += secondary_format

        # Prepare secondary object
        secondary_obj_to_process, secondary_obj_is_duplicated = apply_all_transformations(
            secondary_mesh, "TempDuplicateBounding"
        )

        # Store offset data
        first_vertex_model_1 = len(vertices)
        first_index_model_1 = len(indices)
        config["first_vertex_model_1"] = str(first_vertex_model_1)
        config["first_index_model_1"] = str(first_index_model_1)

        # Extract mesh data
        secondary_vertices, secondary_indices = package_mesh_data(secondary_obj_to_process, secondary_format)
        vertices.extend(secondary_vertices)
        indices.extend(secondary_indices)

        # Get transformation matrices
        matrices.extend(get_matrices(secondary_mesh))

    config[MESH_FORMAT_TAG] = mesh_format

    # Clean up if objects were duplicated
    if primary_obj_is_duplicated and primary_obj_to_process:
        cleanup_duplicated_object(primary_obj_to_process)

    if secondary_obj_is_duplicated and secondary_obj_to_process:
        cleanup_duplicated_object(secondary_obj_to_process)

    # Convert data to ctypes pointers
    vertices_ptr = (Vector3 * len(vertices))(*vertices) if vertices else (Vector3 * 0)()
    indices_ptr = (ctypes.c_size_t * len(indices))(*indices) if indices else (ctypes.c_size_t * 0)()
    matrices_ptr = (ctypes.c_float * len(matrices))(*matrices) if matrices else (ctypes.c_float * 0)()

    # Create StringMap from config
    keys_list = list(config.keys())
    values_list = list(config.values())
    keys_array = (ctypes.c_char_p * len(keys_list))(*[k.encode('utf-8') for k in keys_list])
    values_array = (ctypes.c_char_p * len(values_list))(*[v.encode('utf-8') for v in values_list])
    map_data = StringMap(keys_array, values_array, len(keys_list))

    # Fetch Rust library
    rust_lib = load_latest_dylib()

    # Call Rust function
    rust_result = rust_lib.process_geometry(
        vertices_ptr, len(vertices), indices_ptr, len(indices),
        matrices_ptr, len(matrices), map_data
    )

    # Extract results
    output_vertices = [(vec.x, vec.y, vec.z) for vec in
                       (rust_result.geometry.vertices[i] for i in range(rust_result.geometry.vertex_count))]
    output_indices = [rust_result.geometry.indices[i] for i in range(rust_result.geometry.indices_count)]

    # Extract return options
    output_map = {}
    for i in range(rust_result.map.count):
        key = ctypes.string_at(rust_result.map.keys[i]).decode('utf-8')
        value = ctypes.string_at(rust_result.map.values[i]).decode('utf-8')
        output_map[key] = value

    # Free Rust memory
    rust_lib.free_process_results(rust_result)
    if DEV_MODE:
        ctypes_close_library(rust_lib)

    if "ERROR" in output_map:
        raise HallrException(output_map["ERROR"])

    # Get output mesh format
    output_format = output_map.get(MESH_FORMAT_TAG, primary_format)

    print("python: received config: ", output_map)
    print(f"python: received {len(output_vertices)} vertices: " )
    print(f"python: received {len(output_indices)} indices: " )
    print(f"python: using mesh output_format:{output_format}")

    # Create or update object based on results
    if create_new:
        print("new object new mesh")
        # Create a new object
        new_object = create_object_from_mesh_data(
            output_map, output_vertices, output_indices, output_format,
            output_map.get("model_0_name", "New_Object")
        )
        return new_object
    else:
        print("updating old object with new mesh")
        # Update existing object
        active_obj = bpy.context.view_layer.objects.active
        update_existing_object(active_obj, output_map, output_vertices, output_indices, output_format)
        return None


# Simpler convenience functions that wrap the main processing function

def process_single_mesh(config: Dict[str, str], mesh_obj: bpy.types.Object = None,
                        mesh_format: str = MeshFormat.LINE_CHUNKS,
                        create_new: bool = True) -> Optional[bpy.types.Object]:
    """
    Process a single mesh with the Rust library.

    Args:
        config: Configuration options
        mesh_obj: The mesh object to process
        mesh_format: Mesh format (windows or chunks)
        create_new: Whether to create a new object or modify the active one

    Returns:
        The new object if create_new is True, None otherwise
    """
    return process_mesh_with_rust(
        config,
        primary_mesh=mesh_obj,
        primary_format=mesh_format,
        create_new=create_new
    )

def process_config(config: Dict[str, str]) -> bpy.types.Object:
    """
    Process a command that does not require an input object, only a config.

    Args:
        config: Configuration options

    Returns:
        The new object
    """
    return process_mesh_with_rust(
        config,
        primary_mesh=None,
        secondary_mesh=None,
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


def get_matrices(bpy_object):
    """ Return the world orientation as an array of 16 floats"""
    bm = bpy_object.matrix_world
    return [bm[0][0], bm[0][1], bm[0][2], bm[0][3],
            bm[1][0], bm[1][1], bm[1][2], bm[1][3],
            bm[2][0], bm[2][1], bm[2][2], bm[2][3],
            bm[3][0], bm[3][1], bm[3][2], bm[3][3]]


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
