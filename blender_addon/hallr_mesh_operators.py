"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

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
    angle = math.degrees(math.acos(res))
    return angle


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
                      "origin.")
    bl_options = {'REGISTER', 'UNDO'}

    distance_props: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discretization distance as a percentage of the total AABB length. This value is used when sampling"
                    "parabolic arc edges. Smaller value gives a finer step distance.",
        default=0.005,
        min=0.0001,
        max=4.9999,
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

        config = {"command": "voronoi_mesh",
                  "DISTANCE": str(self.distance_props),
                  }
        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "distance_props")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


# SDF mesh operator
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


# menu containing all tools
class VIEW3D_MT_edit_mesh_hallr_meshtools(bpy.types.Menu):
    bl_label = "Hallr meshtools"

    def draw(self, context):
        layout = self.layout
        layout.operator("mesh.hallr_meshtools_convex_hull_2d")
        layout.operator("mesh.hallr_meshtools_select_end_vertices")
        layout.operator("mesh.hallr_meshtools_select_collinear_edges")
        layout.operator("mesh.hallr_meshtools_select_vertices_until_intersection")
        layout.operator("mesh.hallr_meshtools_select_intersection_vertices")
        layout.operator("mesh.hallr_meshtools_knife_intersect_2d")
        layout.operator("mesh.hallr_meshtools_voronoi_mesh")
        layout.operator("mesh.hallr_meshtools_sdf_mesh_2_5")


# draw function for integration in menus
def menu_func(self, context):
    self.layout.menu("VIEW3D_MT_edit_mesh_hallr_meshtools")
    self.layout.separator()


# define classes for registration
classes = (
    VIEW3D_MT_edit_mesh_hallr_meshtools,
    Hallr_SelectEndVertices,
    Hallr_SelectIntersectionVertices,
    Hallr_SelectVerticesUntilIntersection,
    Hallr_SelectCollinearEdges,
    Hallr_ConvexHull2D,
    Hallr_KnifeIntersect,
    Hallr_Voronoi_Mesh,
    Hallr_SdfMesh25D
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
