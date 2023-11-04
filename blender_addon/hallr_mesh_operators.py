import bpy
import bmesh
import math
import array
from collections import defaultdict
from . import hallr_ffi_utils


class MESH_OT_hallr_knife_intersect(bpy.types.Operator):
    """A knife intersect operator that works in the XY plane, remember to apply any transformations"""

    bl_idname = "mesh.hallr_knife_intersect_2d"
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


class MESH_OT_hallr_convex_hull_2d(bpy.types.Operator):
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

        def process_edge(direction, e):
            to_v = e.verts[direction].index
            for candidate_e in vertex_dict.get(to_v, []):
                if candidate_e.select or candidate_e.index == e.index:
                    continue

                if to_v == candidate_e.verts[0].index:
                    angle = bm.verts[to_v].calc_edge_angle(candidate_e.verts[1])
                elif to_v == candidate_e.verts[1].index:
                    angle = bm.verts[to_v].calc_edge_angle(candidate_e.verts[0])

                if math.degrees(angle) <= angle_criteria:
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


# menu containing all tools
class VIEW3D_MT_edit_mesh_hallr_meshtools(bpy.types.Menu):
    bl_label = "Hallr meshtools"

    def draw(self, context):
        layout = self.layout
        # layout.operator("mesh.hallr_meshtools_knife_intersect")
        layout.operator("mesh.hallr_convex_hull_2d")
        layout.operator("mesh.hallr_meshtools_select_end_vertices")
        layout.operator("mesh.hallr_meshtools_select_collinear_edges")
        layout.operator("mesh.hallr_meshtools_select_vertices_until_intersection")
        layout.operator("mesh.hallr_meshtools_select_intersection_vertices")
        layout.operator("mesh.hallr_knife_intersect_2d")


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
    MESH_OT_hallr_convex_hull_2d,
    MESH_OT_hallr_knife_intersect
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
