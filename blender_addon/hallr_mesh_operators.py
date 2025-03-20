"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import random
import bpy
import bmesh
import math
import array
from collections import defaultdict
from . import hallr_ffi_utils


def angle_between_edges(p0, p1, p2):
    """ angle between the two vectors defined as p0->p1 and p1->p2
    return value in degrees. We can't use vertex.calc_edge_angle() because it only accepts
    vertices only connected to two other vertices (and that is far from the norm in a mesh)"""
    v1 = p1 - p0
    v2 = p2 - p1

    v1mag = math.sqrt(v1.x * v1.x + v1.y * v1.y + v1.z * v1.z)
    if v1mag == 0.0:
        return 0.0

    v1norm = [v1.x / v1mag, v1.y / v1mag, v1.z / v1mag]
    v2mag = math.sqrt(v2.x * v2.x + v2.y * v2.y + v2.z * v2.z)
    if v2mag == 0.0:
        return 0.0

    v2norm = [v2.x / v2mag, v2.y / v2mag, v2.z / v2mag]
    res = v1norm[0] * v2norm[0] + v1norm[1] * v2norm[1] + v1norm[2] * v2norm[2]
    angle = math.degrees(math.acos(max(-1.0, min(res, 1.0))))
    return angle


class Hallr_2DOutline(bpy.types.Operator):
    """Generates the 2d outline from 2D mesh objects"""

    bl_idname = "mesh.hallr_2d_outline"
    bl_label = "Hallr 2D Outline"
    bl_description = ("Outline 2d geometry into a wire frame, the geometry must be flat and on a plane intersecting "
                      "origin")
    bl_options = {'REGISTER', 'UNDO'}

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "2d_outline"}

        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=False)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}


class Hallr_KnifeIntersect(bpy.types.Operator):
    """A knife intersect operator that works in the XY plane, remember to apply any transformations"""

    bl_idname = "mesh.hallr_meshtools_knife_intersect_2d"
    bl_label = "Hallr Knife Intersect 2d"
    bl_options = {'REGISTER', 'UNDO'}

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def execute(self, context):
        active_object = context.active_object
        if active_object is None or active_object.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        if context.mode != 'EDIT_MESH':
            self.report({'ERROR'}, "Must be in edit mode!")
            return {'CANCELLED'}

        # Switch to object mode to gather data without changing the user's selection
        bpy.ops.object.mode_set(mode='OBJECT')

        bpy.context.view_layer.update()

        config = {"command": "knife_intersect"}

        # Call the Rust function

        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, active_object, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(active_object, config_out, vertices, indices)

        # Switch back to edit mode
        bpy.context.view_layer.objects.active = active_object
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}


class Hallr_ConvexHull2D(bpy.types.Operator):
    """A 2D convex hull operator that works in the XY plane, remember to apply any transformations"""

    bl_idname = "mesh.hallr_convex_hull_2d"
    bl_label = "Hallr Convex Hull 2d"
    bl_options = {'REGISTER', 'UNDO'}

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def execute(self, context):
        active_object = context.active_object
        if active_object is None or active_object.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        if context.mode != 'EDIT_MESH':
            self.report({'ERROR'}, "Must be in edit mode!")
            return {'CANCELLED'}

        # Switch to object mode to gather data without changing the user's selection
        bpy.ops.object.mode_set(mode='OBJECT')

        bpy.context.view_layer.update()

        config = {"command": "convex_hull_2d"}

        # Call the Rust function

        vertices, indices, config_out = hallr_ffi_utils.call_rust(config, active_object, only_selected_vertices=True)
        hallr_ffi_utils.handle_windows_line_new_object(vertices, indices)

        # Switch back to edit mode
        bpy.context.view_layer.objects.active = active_object
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}


class Hallr_SimplifyRdp(bpy.types.Operator):
    """Line Simplification using the RDP Algorithm, for 2d and 3d lines"""

    bl_idname = "mesh.hallr_simplify_rdp"
    bl_label = "Hallr Simplify RDP"
    bl_options = {'REGISTER', 'UNDO'}

    simplify_3d_props: bpy.props.BoolProperty(
        name="Simplify 3d",
        description="Simplification will be done in 3d if selected",
        default=True)

    simplify_distance_props: bpy.props.FloatProperty(
        name="Distance",
        description="Discrete distance as a percentage of the longest axis of the model. This value is used for RDP "
                    "simplification.",
        default=0.10,
        min=0.001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "simplify_rdp", "simplify_distance": str(self.simplify_distance_props),
                  "simplify_3d": str(self.simplify_3d_props).lower()}

        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "simplify_distance_props")
        layout.prop(self, "simplify_3d_props")


class Hallr_TriangulateAndFlatten(bpy.types.Operator):
    """Triangulates the mesh and moves all vertices to the XY plane (Z=0)"""
    bl_idname = "mesh.hallr_meshtools_triangulate_and_flatten"
    bl_label = "Triangulate and Flatten XY"
    bl_description = "Triangulates the mesh and moves all vertices to the XY plane (Z=0)"
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

    @classmethod
    def poll(cls, context):
        return context.active_object is not None and context.active_object.type == 'MESH'

    def execute(self, context):
        # Get the active object and mesh
        obj = context.active_object
        me = obj.data

        # Check if we're in Edit Mode
        was_in_edit_mode = context.mode == 'EDIT_MESH'

        # Switch to Object Mode if in Edit Mode
        if was_in_edit_mode:
            bpy.ops.object.mode_set(mode='OBJECT')

        # Apply the object's transformation to the mesh data
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

        # Get a BMesh representation
        bm = bmesh.new()
        bm.from_mesh(me)

        # Triangulate the mesh
        bmesh.ops.triangulate(bm, faces=bm.faces[:])

        # Move all vertices to the XY plane in world space (set Z=0)
        for vert in bm.verts:
            vert.co.z = 0.0

        # Update the mesh with the new data
        bm.to_mesh(me)
        bm.free()

        # Reset the object's transformation (location, rotation, scale)
        obj.location = (0, 0, 0)
        obj.rotation_euler = (0, 0, 0)
        obj.scale = (1, 1, 1)

        # Switch back to Edit Mode if we were originally in Edit Mode
        if was_in_edit_mode:
            bpy.ops.object.mode_set(mode='EDIT')

        # Show the updates in the viewport
        bpy.context.view_layer.update()

        return {'FINISHED'}


class Hallr_SelectEndVertices(bpy.types.Operator):
    """Selects all vertices that are only connected to one other vertex or none (offline plugin)"""
    bl_idname = "mesh.hallr_meshtools_select_end_vertices"
    bl_label = "Select end vertices"
    bl_description = "Selects all vertices that are only connected to one other vertex (offline plugin)"
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def execute(self, context):

        # Get the active mesh
        obj = bpy.context.edit_object
        me = obj.data

        # Get a BMesh representation
        bm = bmesh.from_edit_mesh(me)
        bpy.ops.mesh.select_all(action='DESELECT')

        if len(bm.edges) > 0 or len(bm.faces) > 0:
            vertex_connections = array.array('i', (0 for i in range(0, len(bm.verts))))
            for e in bm.edges:
                for vi in e.verts:
                    vertex_connections[vi.index] += 1
            for f in bm.faces:
                for vi in f.verts:
                    vertex_connections[vi.index] += 1

            for vi in range(0, len(vertex_connections)):
                if vertex_connections[vi] < 2:
                    bm.verts[vi].select = True

        # Show the updates in the viewport
        bmesh.update_edit_mesh(me)

        return {'FINISHED'}


class Hallr_SelectCollinearEdges(bpy.types.Operator):
    """Selects edges that are connected to the selected edges, but limit by an angle constraint.
       You must select at least one edge to get this operation going"""
    bl_idname = "mesh.hallr_meshtools_select_collinear_edges"
    bl_label = "Select collinear edges"
    bl_description = ("Selects edges that are connected to the selected edges, but limit by an angle constraint ("
                      "offline plugin)")
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

    angle_props: bpy.props.FloatProperty(
        name="Angle selection constraint",
        description="selects edges with a relative angle (compared to an already selected edge) smaller than this.",
        default=math.radians(12.0),
        min=math.radians(0.0),
        max=math.radians(180.0),
        precision=6,
        subtype='ANGLE'
    )

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def execute(self, context):

        # Get the active mesh
        obj = bpy.context.edit_object
        me = obj.data

        # Get a BMesh representation
        bm = bmesh.from_edit_mesh(me)
        bm.verts.ensure_lookup_table()
        bm.edges.ensure_lookup_table()
        bm.faces.ensure_lookup_table()

        angle_criteria = math.degrees(self.angle_props)

        vertex_dict = defaultdict(list)  # key by vertex.index to [edges]
        already_selected = set()  # key by edge.index
        work_queue = set()  # edges

        for e in bm.edges:
            vertex_dict[e.verts[0].index].append(e)
            vertex_dict[e.verts[1].index].append(e)
            if e.select:
                work_queue.add(e)

        def process_edge(direction, edge):
            # from_v = edge.verts[0].index if direction == 1 else edge.verts[1].index
            from_v = edge.verts[direction ^ 1].index
            to_v = edge.verts[direction].index
            for candidate_e in vertex_dict.get(to_v, []):
                if candidate_e.select or candidate_e.index == edge.index:
                    continue

                if to_v == candidate_e.verts[0].index:
                    angle = angle_between_edges(bm.verts[from_v].co, bm.verts[to_v].co,
                                                bm.verts[candidate_e.verts[1].index].co)
                    if angle <= angle_criteria:
                        work_queue.add(candidate_e)

                elif to_v == candidate_e.verts[1].index:
                    angle = angle_between_edges(bm.verts[from_v].co, bm.verts[to_v].co,
                                                bm.verts[candidate_e.verts[0].index].co)
                    if angle <= angle_criteria:
                        work_queue.add(candidate_e)

        while len(work_queue) > 0:
            e = work_queue.pop()
            if e.index in already_selected:
                continue

            process_edge(1, e)  # Process edges in one direction
            process_edge(0, e)  # Process edges in the other direction

            e.select = True
            already_selected.add(e.index)

        # Show the updates in the viewport
        bmesh.update_edit_mesh(me)

        return {'FINISHED'}


class Hallr_SelectIntersectionVertices(bpy.types.Operator):
    """Selects all vertices that are connected to more than two other vertices"""
    bl_idname = "mesh.hallr_meshtools_select_intersection_vertices"
    bl_label = "Select intersection vertices"
    bl_description = "Selects all vertices that are connected to more than two other vertices"
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def execute(self, context):

        # Get the active mesh
        obj = bpy.context.edit_object
        me = obj.data

        # Get a BMesh representation
        bm = bmesh.from_edit_mesh(me)
        bpy.ops.mesh.select_all(action='DESELECT')

        if len(bm.edges) > 0 or len(bm.faces) > 0:
            vertex_connections = array.array('i', (0 for i in range(0, len(bm.verts))))
            for e in bm.edges:
                for vi in e.verts:
                    vertex_connections[vi.index] += 1
            for f in bm.faces:
                for vi in f.verts:
                    vertex_connections[vi.index] += 1

            for vi in range(0, len(vertex_connections)):
                if vertex_connections[vi] > 2:
                    bm.verts[vi].select = True

        # Show the updates in the viewport
        bmesh.update_edit_mesh(me)

        return {'FINISHED'}


class Hallr_SelectVerticesUntilIntersection(bpy.types.Operator):
    """Selects all (wire-frame) vertices that are connected to already selected vertices until an intersection is
    detected. The intersection vertex will not be selected."""
    bl_idname = "mesh.hallr_meshtools_select_vertices_until_intersection"
    bl_label = "Select vertices until intersection"
    bl_description = ("Selects all (wire-frame) vertices that are connected to already selected vertices until an "
                      "intersection is detected")
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def execute(self, context):

        # Get the active mesh
        obj = bpy.context.edit_object
        me = obj.data

        # Get a BMesh representation
        bm = bmesh.from_edit_mesh(me)
        bm.verts.ensure_lookup_table()
        bm.edges.ensure_lookup_table()
        bm.faces.ensure_lookup_table()

        if len(bm.edges) > 0 and len(bm.faces) == 0:

            already_selected = set()  # key by vertex.index
            work_queue = set()  # vertex.index

            for v in bm.verts:
                if v.select:
                    work_queue.add(v.index)

            while len(work_queue) > 0:
                v = work_queue.pop()
                if v in already_selected:
                    continue

                if len(bm.verts[v].link_edges) <= 2:
                    bm.verts[v].select = True
                    for e in bm.verts[v].link_edges:
                        if e.verts[0].index != v and e.verts[0].index not in already_selected:
                            work_queue.add(e.verts[0].index)
                        if e.verts[1].index != v and e.verts[1].index not in already_selected:
                            work_queue.add(e.verts[1].index)

                # only mark vertices as already_selected if they've been through the loop once
                already_selected.add(v)

        # Show the updates in the viewport
        bmesh.update_edit_mesh(me)
        return {'FINISHED'}


# Voronoi mesh operator
class Hallr_Voronoi_Mesh(bpy.types.Operator):
    bl_idname = "mesh.hallr_meshtools_voronoi_mesh"
    bl_label = "Voronoi Mesh"
    bl_description = ("Calculate voronoi diagram and add mesh, the geometry must be flat and on a plane intersecting "
                      "origin. It also must be encircled by an outer continuous loop")
    bl_options = {'REGISTER', 'UNDO'}

    distance_props: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discretization distance as a percentage of the total AABB length. This value is used when sampling"
                    "parabolic arc edges. Smaller value gives a finer step distance.",
        default=0.1,
        min=0.0001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    negative_radius_props: bpy.props.BoolProperty(
        name="Negative radius",
        description="Represent voronoi edge distance to input geometry as a negative Z value",
        default=True
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH' and context.mode == 'EDIT_MESH'

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "voronoi_mesh",
                  "DISTANCE": str(self.distance_props),
                  "NEGATIVE_RADIUS": str(self.negative_radius_props).lower(),
                  }
        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "distance_props")
        layout.prop(self, "negative_radius_props")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


# Voronoi operator
class Hallr_Voronoi_Diagram(bpy.types.Operator):
    bl_idname = "mesh.hallr_meshtools_voronoi_diagram"
    bl_label = "Voronoi Diagram"
    bl_description = ("Calculate voronoi diagram, the geometry must be flat and on a plane intersecting "
                      "origin.")
    bl_options = {'REGISTER', 'UNDO'}

    distance_props: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discretization distance as a percentage of the total AABB length. This value is used when sampling"
                    "parabolic arc edges. Smaller value gives a finer step distance.",
        default=0.1,
        min=0.0001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    keep_input_props: bpy.props.BoolProperty(
        name="Keep input edges",
        description="Will keep the input edges in the output",
        default=True
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH' and context.mode == 'EDIT_MESH'

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "voronoi_diagram",
                  "DISTANCE": str(self.distance_props),
                  "KEEP_INPUT": str(self.keep_input_props).lower(),
                  }
        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "distance_props")
        layout.prop(self, "keep_input_props")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


# SDF mesh 2½D operator
class Hallr_SdfMesh25D(bpy.types.Operator):
    """Tooltip: Generate a 3D SDF mesh from 2½D edges."""
    bl_idname = "mesh.hallr_meshtools_sdf_mesh_2_5"
    bl_label = "SDF Mesh 2½D"
    bl_description = (
        "Generate a 3D mesh from 2½D edges. The geometry should be centered on the XY plane intersecting the origin."
        "Each edge is converted into a SDF cone with its endpoint (X, Y) as the tip and Z.abs() as the radius."
        "The resulting mesh will preserve the 2D outline while inflating it based on the median-axis distance."
    )
    bl_options = {'REGISTER', 'UNDO'}

    sdf_divisions_property: bpy.props.IntProperty(
        name="Voxel Divisions",
        description="The longest axis of the model will be divided into this number of voxels; the other axes "
                    "will have a proportionally equal number of voxels.",
        default=100,
        min=50,
        max=600,
        subtype='FACTOR'
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH' and context.mode == 'EDIT_MESH'

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "sdf_mesh_2_5",
                  "SDF_DIVISIONS": str(self.sdf_divisions_property),
                  }
        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "sdf_divisions_property")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


# SDF mesh operator
class Hallr_SdfMesh(bpy.types.Operator):
    """Generate a 3D SDF mesh from 3d edges."""
    bl_idname = "mesh.hallr_meshtools_sdf_mesh"
    bl_label = "SDF Mesh"
    bl_description = (
        "Generate a 3D mesh from 3D edges."
        "Each edge is converted into a SDF tube with a predefined radius."
    )
    bl_options = {'REGISTER', 'UNDO'}

    sdf_divisions_prop: bpy.props.IntProperty(
        name="Voxel Divisions",
        description="The longest axis of the model will be divided into this number of voxels; the other axes "
                    "will have a proportionally equal number of voxels.",
        default=100,
        min=50,
        max=600,
        subtype='FACTOR'
    )

    sdf_radius_prop: bpy.props.FloatProperty(
        name="Radius",
        description="Voxel tube radius as a percentage of the total AABB",
        default=1.0,
        min=0.01,
        max=19.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH' and context.mode == 'EDIT_MESH'

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "sdf_mesh",
                  "SDF_DIVISIONS": str(self.sdf_divisions_prop),
                  "SDF_RADIUS_MULTIPLIER": str(self.sdf_radius_prop)
                  }

        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "sdf_divisions_prop")
        layout.prop(self, "sdf_radius_prop")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


# Random vertices operation
class Hallr_RandomVertices(bpy.types.Operator):
    """Generate some random vertices"""
    bl_idname = "mesh.hallr_meshtools_random_vertices"
    bl_label = "Random vertices"
    bl_description = (
        "Generate some random vertices in the XY plane"
    )
    bl_options = {'REGISTER', 'UNDO'}

    number_of_vertices_prop: bpy.props.IntProperty(
        name="Number of vertices",
        description="Generate this many vertices",
        default=10,
        min=1,
        max=100,
    )

    std_deviation_prop: bpy.props.FloatProperty(
        name="Standard deviation",
        description="Defines the random spread of the vertices",
        default=1.0,
        min=0.01,
        max=19.9999,
        precision=6,
    )

    merge_distance_prop: bpy.props.FloatProperty(
        name="Merge Distance",
        description="Merge vertices that are closer than this distance",
        default=0.01,
        min=0.0,
        max=2.0,
        precision=6,
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH' and context.mode == 'EDIT_MESH'

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Get the mesh data
        mesh = bpy.context.edit_object.data
        bm = bmesh.from_edit_mesh(mesh)

        # Create random vertices
        for i in range(self.number_of_vertices_prop):
            angle = random.uniform(0, 2 * math.pi)
            radius = random.gauss(0, self.std_deviation_prop)

            x = radius * math.cos(angle)
            y = radius * math.sin(angle)

            # Add vertex to the mesh
            vert = bm.verts.new((x, y, 0.0))

        # Select only the generated vertices
        bpy.ops.mesh.select_all(action='DESELECT')
        for vert in bm.verts:
            if vert.index == -1:
                # this vertex was randomly generated
                vert.select = True

        # Merge vertices based on distance
        bpy.ops.mesh.remove_doubles(threshold=self.merge_distance_prop)

        # Update the mesh
        bmesh.update_edit_mesh(mesh)
        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "number_of_vertices_prop")
        layout.prop(self, "std_deviation_prop")
        layout.prop(self, "merge_distance_prop")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


# Discretize operator
class Hallr_Discretize(bpy.types.Operator):
    """Subdivide edges by length"""
    bl_idname = "mesh.hallr_meshtools_discretize"
    bl_label = "Subdivide by length"
    bl_description = (
        "Subdivides edges by length."
    )
    bl_options = {'REGISTER', 'UNDO'}

    discretize_length_prop: bpy.props.FloatProperty(
        name="Length",
        description="Discretize length as a percentage of the total AABB. The edges will be split up by up to this "
                    "length, and no more",
        default=25.0,
        min=0.1,
        max=51,
        precision=3,
        subtype='PERCENTAGE'
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH' and context.mode == 'EDIT_MESH'

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "discretize",
                  "discretize_length": str(self.discretize_length_prop),
                  }

        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "discretize_length_prop")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


class Hallr_Centerline(bpy.types.Operator):
    """Finds the center line of closed geometry, works in the XY plane"""

    bl_idname = "mesh.hallr_centerline"
    bl_label = "Hallr 2D Centerline"
    bl_options = {'REGISTER', 'UNDO'}

    angle_props: bpy.props.FloatProperty(
        name="Angle",
        description="Edge rejection angle, edges with edge-to-segment angles larger than this will be rejected",
        default=math.radians(89.0),
        min=math.radians(0.000001),
        max=math.radians(89.999999),
        precision=6,
        subtype='ANGLE',
    )

    weld_props: bpy.props.BoolProperty(
        name="Weld the centerline to outline",
        description="Centerline and outline will share vertices if they intersect",
        default=True
    )

    keep_input_props: bpy.props.BoolProperty(
        name="Keep input edges",
        description="Will keep the input edges in the output, will override the weld property if inactive",
        default=True
    )

    negative_radius_props: bpy.props.BoolProperty(
        name="Negative radius",
        description="Represent voronoi edge distance to input geometry as a negative Z value",
        default=True
    )

    remove_internals_props: bpy.props.BoolProperty(
        name="Remove internal edges",
        description="Remove edges internal to islands for the geometry. I.e. it will remove geometry generated from "
                    "closed loops inside closed loops",
        default=True
    )

    distance_props: bpy.props.FloatProperty(
        name="Distance",
        description="Discrete distance as a percentage of the AABB. This value is used when sampling parabolic arc "
                    "edges. It is also used for RDP simplification.",
        default=0.05,
        min=0.001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    simplify_props: bpy.props.BoolProperty(
        name="Simplify line strings",
        description="Simplify voronoi edges connected as in a line string. The 'distance' property is used.",
        default=True
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH'

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "centerline",
                  "ANGLE": str(math.degrees(self.angle_props)),
                  "REMOVE_INTERNALS"
                  : str(self.remove_internals_props).lower(),
                  "KEEP_INPUT"
                  : str(self.keep_input_props).lower(),
                  "NEGATIVE_RADIUS"
                  : str(self.negative_radius_props).lower(),
                  "DISTANCE"
                  : str(self.distance_props),
                  "SIMPLIFY"
                  : str(self.simplify_props).lower(),
                  "WELD"
                  : str(self.weld_props).lower(),
                  }
        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "angle_props")
        if self.keep_input_props:
            layout.prop(self, "weld_props")
        layout.prop(self, "keep_input_props")
        layout.prop(self, "negative_radius_props")
        layout.prop(self, "remove_internals_props")
        if self.simplify_props:
            layout.prop(self, "distance_props")
        layout.prop(self, "simplify_props")


# menu containing all tools
class VIEW3D_MT_edit_mesh_hallr_meshtools(bpy.types.Menu):
    bl_label = "Hallr meshtools"

    def draw(self, context):
        layout = self.layout
        layout.operator("mesh.hallr_meshtools_triangulate_and_flatten")
        layout.operator("mesh.hallr_2d_outline")
        layout.operator("mesh.hallr_meshtools_select_end_vertices")
        layout.operator("mesh.hallr_meshtools_select_collinear_edges")
        layout.operator("mesh.hallr_convex_hull_2d")
        layout.operator("mesh.hallr_meshtools_select_vertices_until_intersection")
        layout.operator("mesh.hallr_meshtools_select_intersection_vertices")
        layout.operator("mesh.hallr_meshtools_knife_intersect_2d")
        layout.operator("mesh.hallr_meshtools_voronoi_mesh")
        layout.operator("mesh.hallr_meshtools_voronoi_diagram")
        layout.operator("mesh.hallr_meshtools_sdf_mesh_2_5")
        layout.operator("mesh.hallr_meshtools_sdf_mesh")
        layout.operator("mesh.hallr_simplify_rdp")
        layout.operator("mesh.hallr_centerline")
        layout.operator("mesh.hallr_meshtools_discretize")
        layout.operator("mesh.hallr_meshtools_random_vertices")


# draw function for integration in menus
def menu_func(self, context):
    self.layout.menu("VIEW3D_MT_edit_mesh_hallr_meshtools")
    self.layout.separator()


# define classes for registration
classes = (
    VIEW3D_MT_edit_mesh_hallr_meshtools,
    Hallr_TriangulateAndFlatten,
    Hallr_SelectEndVertices,
    Hallr_SelectIntersectionVertices,
    Hallr_SelectVerticesUntilIntersection,
    Hallr_SelectCollinearEdges,
    Hallr_ConvexHull2D,
    Hallr_KnifeIntersect,
    Hallr_Voronoi_Mesh,
    Hallr_Voronoi_Diagram,
    Hallr_SdfMesh25D,
    Hallr_SdfMesh,
    Hallr_2DOutline,
    Hallr_SimplifyRdp,
    Hallr_Centerline,
    Hallr_Discretize,
    Hallr_RandomVertices,
)


# registering and menu integration
def register():
    for cls in classes:
        bpy.utils.register_class(cls)
    bpy.types.VIEW3D_MT_edit_mesh_context_menu.prepend(menu_func)


# unregistering and removing menus
def unregister():
    for cls in reversed(classes):
        try:
            bpy.utils.unregister_class(cls)
        except (RuntimeError, NameError):
            pass
    bpy.types.VIEW3D_MT_edit_mesh_context_menu.remove(menu_func)
