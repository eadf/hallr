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
import mathutils

DEV_MODE = False  # Set this to False for distribution
if DEV_MODE:
    import hallr_ffi_utils
    from hallr_mesh_operators.common import BaseOperatorMixin
else:
    from .. import hallr_ffi_utils
    from ..hallr_mesh_operators.common import BaseOperatorMixin


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


def get_rotation_to(from_vn, to_vn):
    """Returns the rotation quaternion needed when rotating fromVn to toVn. fromVn and toVn should be normalized."""
    if from_vn[0] == to_vn[0] and from_vn[1] == to_vn[1] and from_vn[2] == to_vn[2]:
        return mathutils.Quaternion((1.0, 0, 0, 0))
    cross_product = from_vn.cross(to_vn)
    cross_product.normalize()
    angle = math.acos(from_vn.dot(to_vn))
    return mathutils.Quaternion(cross_product, angle)


class MESH_OT_hallr_meta_volume(bpy.types.Operator, BaseOperatorMixin):
    """Volumetric edge fill using meta capsules"""
    bl_idname = "mesh.hallr_metavolume"
    bl_label = "Hallr Metavolume from edges"
    bl_icon = "MESH_ICOSPHERE"
    bl_description = 'Generates volume from a lattice of edges using metaballs (pure blender/python operator)'
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

    CAPSULE_VECTOR = mathutils.Vector((1.0, 0.0, 0.0))  # capsule orientation

    radius_prop: bpy.props.FloatProperty(name="Radius", default=1.0, min=0.0001, max=1000,
                                         description="Radius of the meta capsules")
    resolution_prop: bpy.props.FloatProperty(name="Resolution", default=0.25, min=0.05, max=1,
                                             description="Resolution of the meta capsules")
    threshold_prop: bpy.props.FloatProperty(name="Threshold", default=0.05, min=0.001, max=1.99999,
                                            description="Threshold of the meta capsules")
    convert_to_mesh_prop: bpy.props.BoolProperty(name="Convert to mesh", default=False,
                                                 description="Convert the metaballs to mesh directly")

    # Store the source object name to retrieve it during redo operations
    source_object_name: bpy.props.StringProperty()

    original_matrix = None

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def new_capsule(self, meta_factory, v0, v1, radius):
        segment = v1 - v0
        capsule = meta_factory.new()
        capsule.co = (v1 + v0) / 2.0
        capsule.type = 'CAPSULE'
        capsule.radius = radius
        capsule.size_x = segment.length / 2.0
        direction = segment.normalized()
        quaternion = get_rotation_to(self.CAPSULE_VECTOR, direction)
        capsule.rotation = quaternion
        return capsule

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.prop(self, "radius_prop")
        row = layout.row()
        row.prop(self, "resolution_prop")
        row = layout.row()
        row.prop(self, "threshold_prop")
        row = layout.row()
        row.prop(self, "convert_to_mesh_prop")

    def invoke(self, context, event):
        # This runs before the first execution, capturing the source object
        source_obj = context.active_object
        self.source_object_name = source_obj.name

        # Store the original matrix directly
        self.original_matrix = source_obj.matrix_world.copy()

        return self.execute(context)

    def execute(self, context):

        # Get the source object by name, which works even during redo operations
        source_obj = bpy.data.objects.get(self.source_object_name)
        if not source_obj:
            self.report({'ERROR'}, f"Source object '{self.source_object_name}' not found")
            return {'CANCELLED'}

        # Create the bmesh from the source object
        source_bm = bmesh.new()
        source_bm.from_mesh(source_obj.data)

        # Remove any existing metaball objects with the same name
        meta_name = f"Metavolume_{source_obj.name}"
        for obj in bpy.data.objects:
            if obj.name.startswith(meta_name):
                bpy.data.objects.remove(obj)

        # Clean up old metaballs to avoid data blocks accumulation
        for mb in bpy.data.metaballs:
            if mb.name.startswith(meta_name) and mb.users == 0:
                bpy.data.metaballs.remove(mb)

        # Create new metaball
        mball = bpy.data.metaballs.new(meta_name)
        mball.resolution = self.resolution_prop
        mball.threshold = self.threshold_prop

        # Create a custom undo step - this will just delete the new object when undone
        bpy.ops.ed.undo_push(message="Create New Object")

        meta_obj = bpy.data.objects.new(meta_name, mball)

        for edge in source_bm.edges:
            fromV = mathutils.Vector(edge.verts[0].co)
            toV = mathutils.Vector(edge.verts[1].co)
            self.new_capsule(mball.elements, fromV, toV, self.radius_prop)

        bpy.context.scene.collection.objects.link(meta_obj)

        # Set the meta object's world matrix
        if self.original_matrix is not None:
            meta_obj.matrix_world = self.original_matrix

        if self.convert_to_mesh_prop:
            # Deselect all objects
            for obj in bpy.context.selected_objects:
                obj.select_set(False)

            # Select and make the metaball object active
            meta_obj.select_set(True)
            bpy.context.view_layer.objects.active = meta_obj

            # Convert metaball to mesh
            bpy.ops.object.convert(target='MESH')

            # Rename the converted object
            converted_obj = bpy.context.active_object
            converted_obj.name = f"Volume_{source_obj.name}"
        else:
            # Just select the metaball object
            source_obj.select_set(False)
            meta_obj.select_set(True)
            bpy.context.view_layer.objects.active = meta_obj

        source_bm.free()
        return {'FINISHED'}


class MESH_OT_hallr_triangulate_and_flatten(bpy.types.Operator, BaseOperatorMixin):
    """Triangulates the mesh and moves all vertices to the XY plane (Z=0)"""
    bl_idname = "mesh.hallr_meshtools_triangulate_and_flatten"
    bl_icon = "MOD_TRIANGULATE"
    bl_label = "Triangulate and Flatten XY"
    bl_description = "Triangulates the mesh and moves all vertices to the XY plane (Z=0)"
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

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


class MESH_OT_hallr_select_end_vertices(bpy.types.Operator, BaseOperatorMixin):
    """Selects all vertices that are only connected to one other vertex or none (blender/python plugin)"""
    bl_idname = "mesh.hallr_meshtools_select_end_vertices"
    bl_label = "Select end vertices"
    bl_icon = "EVENT_END"
    bl_description = "Selects all vertices that are only connected to one other vertex (blender/python plugin)"
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

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


class MESH_OT_hallr_select_collinear_edges(bpy.types.Operator, BaseOperatorMixin):
    """Selects edges that are connected to the selected edges, but limit by an angle constraint.
       You must select at least one edge to get this operation going"""
    bl_idname = "mesh.hallr_meshtools_select_collinear_edges"
    bl_icon = "SNAP_EDGE"
    bl_label = "Select collinear edges"
    bl_description = ("Selects edges that are connected to the selected edges, but limit by an angle constraint ("
                      "blender/python plugin)")
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

    angle_prop: bpy.props.FloatProperty(
        name="Angle selection constraint",
        description="selects edges with a relative angle (compared to an already selected edge) smaller than this.",
        default=math.radians(12.0),
        min=math.radians(0.0),
        max=math.radians(180.0),
        precision=6,
        subtype='ANGLE'
    )

    def execute(self, context):

        # Get the active mesh
        obj = bpy.context.edit_object
        me = obj.data

        # Get a BMesh representation
        bm = bmesh.from_edit_mesh(me)
        bm.verts.ensure_lookup_table()
        bm.edges.ensure_lookup_table()
        bm.faces.ensure_lookup_table()

        angle_criteria = math.degrees(self.angle_prop)

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


class MESH_OT_hallr_select_intersection_vertices(bpy.types.Operator, BaseOperatorMixin):
    """Selects all vertices that are connected to more than two other vertices"""
    bl_idname = "mesh.hallr_meshtools_select_intersection_vertices"
    bl_icon = "SELECT_INTERSECT"
    bl_label = "Select intersection vertices"
    bl_description = "Selects all vertices that are connected to more than two other vertices"
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

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


class MESH_OT_hallr_select_vertices_until_intersection(bpy.types.Operator, BaseOperatorMixin):
    """Selects all (wire-frame) vertices that are connected to already selected vertices until an intersection is
    detected. The intersection vertex will not be selected."""
    bl_idname = "mesh.hallr_meshtools_select_vertices_until_intersection"
    bl_icon = "GP_SELECT_POINTS"
    bl_label = "Select vertices until intersection"
    bl_description = ("Selects all (wire-frame) vertices that are connected to already selected vertices until an "
                      "intersection is detected")
    bl_options = {'REGISTER', 'UNDO'}  # enable undo for the operator.

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


# Random vertices operation
class MESH_OT_hallr_random_vertices(bpy.types.Operator, BaseOperatorMixin):
    """Generate some random vertices in the XY plane"""
    bl_idname = "mesh.hallr_meshtools_random_vertices"
    bl_label = "[XY] Random vertices"
    bl_icon = "OUTLINER_OB_POINTCLOUD"
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
        subtype='UNSIGNED'
    )

    std_deviation_prop: bpy.props.FloatProperty(
        name="Standard deviation",
        description="Defines the random spread of the vertices",
        default=1.0,
        min=0.01,
        max=19.9999,
        precision=6,
        subtype='UNSIGNED'
    )

    remove_doubles_threshold_prop: bpy.props.FloatProperty(
        name="Merge Distance",
        description="Maximum distance between vertices to be merged",
        default=0.001,
        min=0.000001,
        max=0.01,
        precision=6,
        unit='LENGTH'
    )

    use_remove_doubles_prop: bpy.props.BoolProperty(
        name="Use remove doubled",
        description="Activates the remove doubles feature",
        default=True
    )

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

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
        if self.use_remove_doubles_prop:
            with hallr_ffi_utils.timer("Python: bpy.ops.mesh.remove_doubles()"):
                bpy.ops.mesh.remove_doubles(threshold=self.remove_doubles_threshold_prop)

        # Update the mesh
        bmesh.update_edit_mesh(mesh)
        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "number_of_vertices_prop")
        layout.prop(self, "std_deviation_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop
