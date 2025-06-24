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


class BaseOperatorMixin:
    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob is not None and ob.type == 'MESH' and context.mode == 'EDIT_MESH'


def get_rotation_to(from_vn, to_vn):
    """Returns the rotation quaternion needed when rotating fromVn to toVn. fromVn and toVn should be normalized."""
    if from_vn[0] == to_vn[0] and from_vn[1] == to_vn[1] and from_vn[2] == to_vn[2]:
        return mathutils.Quaternion((1.0, 0, 0, 0))
    cross_product = from_vn.cross(to_vn)
    cross_product.normalize()
    angle = math.acos(from_vn.dot(to_vn))
    return mathutils.Quaternion(cross_product, angle)


class MESH_OT_hallr_meta_volume(bpy.types.Operator):
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


class MESH_OT_hallr_2d_outline(bpy.types.Operator, BaseOperatorMixin):
    """Generates the 2d outline from 2D mesh objects"""

    bl_idname = "mesh.hallr_2d_outline"
    bl_icon = "MOD_OUTLINE"
    bl_label = "[XY] Hallr 2D Outline"
    bl_description = ("Outline 2d geometry into a wire frame, the geometry *must* be flat (Z=0) and on the XY plane"
                      "Typically the kind of mesh you get when you convert a text to mesh.")
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        obj = context.active_object

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {hallr_ffi_utils.COMMAND_TAG: "2d_outline"}

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.TRIANGULATED,
                                                create_new=False)
        except Exception as e:
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}


class MESH_OT_hallr_knife_intersect(bpy.types.Operator, BaseOperatorMixin):
    """A knife intersect operator that works in the XY plane, remember to apply any transformations"""

    bl_idname = "mesh.hallr_meshtools_knife_intersect_2d"
    bl_label = "[XY] Hallr Knife Intersect 2d"
    bl_icon = "INTERNET_OFFLINE"
    bl_options = {'REGISTER', 'UNDO'}
    bl_description = (
        "Finds and cuts intersections between edges in the XY plane. "
        "Creates new vertices at intersection points. "
        "Ensure mesh transformations are applied before use."
    )

    def execute(self, context):
        obj = context.active_object

        # Switch to object mode to gather data without changing the user's selection
        bpy.ops.object.mode_set(mode='OBJECT')

        bpy.context.view_layer.update()

        config = {hallr_ffi_utils.COMMAND_TAG: "knife_intersect"}

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.EDGES,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}


class MESH_OT_hallr_convex_hull_2d(bpy.types.Operator, BaseOperatorMixin):
    """A 2D convex hull operator that works in the XY plane, remember to apply any transformations"""

    bl_idname = "mesh.hallr_convex_hull_2d"
    bl_label = "[XY] Hallr Convex Hull 2d"
    bl_icon = "MESH_CAPSULE"
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        obj = context.active_object

        # Switch to object mode to gather data without changing the user's selection
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {hallr_ffi_utils.COMMAND_TAG: "convex_hull_2d"}

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.POINT_CLOUD,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}


class MESH_OT_hallr_simplify_rdp(bpy.types.Operator, BaseOperatorMixin):
    """Line Simplification using the RDP Algorithm, for 2d and 3d lines"""

    bl_idname = "mesh.hallr_simplify_rdp"
    bl_icon = 'OUTLINER_DATA_CURVE'
    bl_label = "Hallr Simplify RDP"
    bl_options = {'REGISTER', 'UNDO'}

    simplify_3d_prop: bpy.props.BoolProperty(
        name="Simplify 3d",
        description="Simplification will be done in 3d if selected",
        default=True)

    simplify_distance_prop: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discrete distance as a percentage of the longest axis of the model. This value is used for RDP "
                    "simplification.",
        default=0.10,
        min=0.001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def execute(self, context):
        obj = context.active_object

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {hallr_ffi_utils.COMMAND_TAG: "simplify_rdp", "simplify_distance": str(self.simplify_distance_prop),
                  "simplify_3d": str(self.simplify_3d_prop).lower()}

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.EDGES,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.label(icon='FIXED_SIZE')
        row.prop(self, "simplify_distance_prop")
        layout.prop(self, "simplify_3d_prop")


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


# Voronoi mesh operator
class MESH_OT_hallr_voroni_mesh(bpy.types.Operator, BaseOperatorMixin):
    bl_idname = "mesh.hallr_meshtools_voronoi_mesh"
    bl_label = "[XY] Voronoi Mesh"
    bl_icon = "MESH_UVSPHERE"
    bl_description = ("Calculate voronoi diagram and add mesh, the geometry must be flat and on a plane intersecting "
                      "origin. It also must be encircled by an outer continuous loop")
    bl_options = {'REGISTER', 'UNDO'}

    distance_prop: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discretization distance as a percentage of the total AABB length. This value is used when sampling"
                    "parabolic arc edges. Smaller value gives a finer step distance.",
        default=0.1,
        min=0.0001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    negative_radius_prop: bpy.props.BoolProperty(
        name="Negative radius",
        description="Represent voronoi edge distance to input geometry as a negative Z value",
        default=True
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

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {hallr_ffi_utils.COMMAND_TAG: "voronoi_mesh",
                  "DISTANCE": str(self.distance_prop),
                  "NEGATIVE_RADIUS": str(self.negative_radius_prop).lower(),
                  }
        if self.use_remove_doubles_prop:
            config[hallr_ffi_utils.VERTEX_MERGE_TAG] = str(self.remove_doubles_threshold_prop)

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.EDGES,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.label(icon='FIXED_SIZE')
        row.prop(self, "distance_prop")
        layout.prop(self, "negative_radius_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop


# Voronoi operator
class MESH_OT_hallr_voronoi_diagram(bpy.types.Operator, BaseOperatorMixin):
    bl_idname = "mesh.hallr_meshtools_voronoi_diagram"
    bl_label = "[XY] Voronoi Diagram"
    bl_icon = "CURVE_NCURVE"
    bl_description = ("Calculate voronoi diagram, the geometry must be flat and on a plane intersecting "
                      "origin.")
    bl_options = {'REGISTER', 'UNDO'}

    distance_prop: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discretization distance as a percentage of the total AABB length. This value is used when sampling"
                    "parabolic arc edges. Smaller value gives a finer step distance.",
        default=0.1,
        min=0.0001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    keep_input_prop: bpy.props.BoolProperty(
        name="Keep input edges",
        description="Will keep the input edges in the output",
        default=True
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

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {hallr_ffi_utils.COMMAND_TAG: "voronoi_diagram",
                  "DISTANCE": str(self.distance_prop),
                  "KEEP_INPUT": str(self.keep_input_prop).lower(),
                  }
        if self.use_remove_doubles_prop:
            config[hallr_ffi_utils.VERTEX_MERGE_TAG] = str(self.remove_doubles_threshold_prop)

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.EDGES,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.label(icon='FIXED_SIZE')
        row.prop(self, "distance_prop")
        layout.prop(self, "keep_input_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop


# SDF mesh 2½D operator
class MESH_OT_hallr_sdf_mesh_25D(bpy.types.Operator, BaseOperatorMixin):
    """Tooltip: Generate a 3D SDF mesh from 2½D edges."""
    bl_idname = "mesh.hallr_meshtools_sdf_mesh_2_5"
    bl_label = "SDF Mesh 2½D"
    bl_icon = "MESH_CONE"
    bl_description = (
        "Generate a 3D mesh from 2½D edges. Typically this operation works on the data generated from the centerline operation."
        "The geometry should placed on the XY plane intersecting the origin."
        "Each edge is converted into a SDF cone with its endpoint (X, Y) as the tip and Z.abs() as the radius."
        "The resulting mesh will preserve the 2D outline while inflating it based on the median-axis distance."
    )
    bl_options = {'REGISTER', 'UNDO'}

    sdf_divisions_prop: bpy.props.IntProperty(
        name="Voxel Divisions",
        description="The longest axis of the model will be divided into this number of voxels; the other axes "
                    "will have a proportionally equal number of voxels.",
        default=100,
        min=50,
        max=600,
        subtype='UNSIGNED'
    )

    sdf_radius_multiplier_prop: bpy.props.FloatProperty(
        name="Radius multiplier",
        description="Radius multiplier",
        default=1.0,
        min=0.01,
        max=5.0,
        precision=6,
    )

    backend_variant_items = (
        ("sdf_mesh_2½_fsn", "Fast Surface Nets", "use fast_surface_nets backend"),
        ("sdf_mesh_2½_saft", "Saft", "use saft backend"),
    )

    cmd_backend_prop: bpy.props.EnumProperty(name="Backend", items=backend_variant_items, default="sdf_mesh_2½_fsn")

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

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {hallr_ffi_utils.COMMAND_TAG: self.cmd_backend_prop,
                  "SDF_DIVISIONS": str(self.sdf_divisions_prop),
                  "SDF_RADIUS_MULTIPLIER": str(self.sdf_radius_multiplier_prop),
                  }
        if self.use_remove_doubles_prop:
            config[hallr_ffi_utils.VERTEX_MERGE_TAG] = str(self.remove_doubles_threshold_prop)
        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.EDGES,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.label(icon='DRIVER_DISTANCE')
        row.prop(self, "sdf_divisions_prop")
        row = layout.row()
        row.prop(self, "sdf_radius_multiplier_prop")
        row = layout.row()
        row.prop(self, "cmd_backend_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop


# SDF mesh operator
class MESH_OT_hallr_sdf_mesh(bpy.types.Operator, BaseOperatorMixin):
    """Generate a 3D SDF mesh from 3d edges."""
    bl_idname = "mesh.hallr_meshtools_sdf_mesh"
    bl_label = "SDF Mesh"
    bl_icon = "MESH_ICOSPHERE"
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
        subtype='UNSIGNED'
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

    backend_variant_items = (
        ("sdf_mesh", "Fast Surface Nets", "use fast_surface_nets backend"),
        ("sdf_mesh_saft", "Saft", "use saft backend"),
    )
    cmd_backend_prop: bpy.props.EnumProperty(name="Backend", items=backend_variant_items, default="sdf_mesh")

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

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {hallr_ffi_utils.COMMAND_TAG: self.cmd_backend_prop,
                  "SDF_DIVISIONS": str(self.sdf_divisions_prop),
                  "SDF_RADIUS_MULTIPLIER": str(self.sdf_radius_prop),
                  }
        if self.use_remove_doubles_prop:
            config[hallr_ffi_utils.VERTEX_MERGE_TAG] = str(self.remove_doubles_threshold_prop)
        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.EDGES,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.label(icon='DRIVER_DISTANCE')
        row.prop(self, "sdf_divisions_prop")
        row = layout.row()
        row.prop(self, "sdf_radius_prop")
        row = layout.row()
        row.prop(self, "cmd_backend_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop


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
            with timer("Python: bpy.ops.mesh.remove_doubles()"):
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


# Discretize operator
class MESH_OT_hallr_discretize(bpy.types.Operator, BaseOperatorMixin):
    """Subdivide edges by length"""
    bl_idname = "mesh.hallr_meshtools_discretize"
    bl_label = "Subdivide by length"
    bl_icon = "CENTER_ONLY"
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

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def execute(self, context):
        obj = context.active_object

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {hallr_ffi_utils.COMMAND_TAG: "discretize",
                  "discretize_length": str(self.discretize_length_prop),
                  }

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.EDGES,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.label(icon='FIXED_SIZE')
        row.prop(self, "discretize_length_prop")


class MESH_OT_hallr_centerline(bpy.types.Operator, BaseOperatorMixin):
    """Finds the center line of closed geometry, works in the XY plane"""

    bl_idname = "mesh.hallr_centerline"
    bl_icon = "CONE"
    bl_label = "[XY] Hallr 2D Centerline"
    bl_options = {'REGISTER', 'UNDO'}

    angle_prop: bpy.props.FloatProperty(
        name="Angle",
        description="Edge rejection angle, edges with edge-to-segment angles larger than this will be rejected",
        default=math.radians(89.0),
        min=math.radians(0.000001),
        max=math.radians(89.999999),
        precision=6,
        subtype='ANGLE',
    )

    keep_input_prop: bpy.props.BoolProperty(
        name="Keep input edges",
        description="Will keep the input edges in the output",
        default=True
    )

    negative_radius_prop: bpy.props.BoolProperty(
        name="Negative radius",
        description="Represent voronoi edge distance to input geometry as a negative Z value",
        default=True
    )

    remove_internals_prop: bpy.props.BoolProperty(
        name="Remove internal edges",
        description="Remove edges internal to islands for the geometry. I.e. it will remove geometry generated from "
                    "closed loops inside closed loops",
        default=True
    )

    distance_prop: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discrete distance as a percentage of the AABB. This value is used when sampling parabolic arc "
                    "edges. It is also used for RDP simplification.",
        default=0.05,
        min=0.001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    simplify_prop: bpy.props.BoolProperty(
        name="Simplify line strings",
        description="RDP Simplify voronoi edges connected as in a line string. The 'distance' property is used.",
        default=True
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

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {hallr_ffi_utils.COMMAND_TAG: "centerline",
                  "ANGLE": str(math.degrees(self.angle_prop)),
                  "REMOVE_INTERNALS"
                  : str(self.remove_internals_prop).lower(),
                  "KEEP_INPUT"
                  : str(self.keep_input_prop).lower(),
                  "NEGATIVE_RADIUS"
                  : str(self.negative_radius_prop).lower(),
                  "DISTANCE"
                  : str(self.distance_prop),
                  "SIMPLIFY"
                  : str(self.simplify_prop).lower(),
                  }
        if self.use_remove_doubles_prop:
            config[hallr_ffi_utils.VERTEX_MERGE_TAG] = str(self.remove_doubles_threshold_prop)
        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.EDGES,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.label(icon='FILTER')
        row.prop(self, "angle_prop")
        layout.prop(self, "keep_input_prop")
        layout.prop(self, "negative_radius_prop")
        layout.prop(self, "remove_internals_prop")
        row = layout.row()
        row.label(icon='FIXED_SIZE')
        row.prop(self, "distance_prop")
        layout.prop(self, "simplify_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop


class MESH_OT_hallr_mesh_cleanup(bpy.types.Operator, BaseOperatorMixin):
    bl_idname = "mesh.hallr_meshtools_mesh_cleanup"
    bl_label = "Mesh cleanup"
    bl_icon = 'MOD_MESHDEFORM'
    bl_description = "Try to fix a non-manifold mesh"
    bl_options = {'REGISTER', 'UNDO'}

    iterations_count_prop: bpy.props.IntProperty(
        name="Max iterations",
        description="Maximum number of iterations for remeshing. Increase this if your remeshed mesh contains irregularities."
                    "Higher values improve mesh quality but increase computation time.",
        default=10,
        min=1,
        max=100
    )

    def invoke(self, context, event):
        self.manifold_not_checked = True
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def execute(self, context):
        obj = context.active_object

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {
            hallr_ffi_utils.COMMAND_TAG: "mesh_cleanup",
            "max_iterations": str(self.iterations_count_prop),
        }

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=hallr_ffi_utils.MeshFormat.TRIANGULATED,
                                                create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        box = layout.box()
        row = box.row()
        row.prop(self, "iterations_count_prop")


# menu containing all tools
class VIEW3D_MT_edit_mesh_hallr_meshtools(bpy.types.Menu):
    bl_label = "Hallr meshtools"

    def draw(self, context):
        layout = self.layout
        layout.operator(MESH_OT_hallr_2d_outline.bl_idname, icon=MESH_OT_hallr_2d_outline.bl_icon)
        layout.operator(MESH_OT_hallr_convex_hull_2d.bl_idname, icon=MESH_OT_hallr_convex_hull_2d.bl_icon)
        layout.operator(MESH_OT_hallr_voroni_mesh.bl_idname, icon=MESH_OT_hallr_voroni_mesh.bl_icon)
        layout.operator(MESH_OT_hallr_voronoi_diagram.bl_idname, icon=MESH_OT_hallr_voronoi_diagram.bl_icon)
        layout.operator(MESH_OT_hallr_sdf_mesh_25D.bl_idname, icon=MESH_OT_hallr_sdf_mesh_25D.bl_icon)
        layout.operator(MESH_OT_hallr_sdf_mesh.bl_idname, icon=MESH_OT_hallr_sdf_mesh.bl_icon)
        layout.operator(MESH_OT_hallr_meta_volume.bl_idname, icon=MESH_OT_hallr_sdf_mesh.bl_icon)
        layout.operator(MESH_OT_hallr_simplify_rdp.bl_idname, icon=MESH_OT_hallr_simplify_rdp.bl_icon)
        layout.operator(MESH_OT_hallr_centerline.bl_idname, icon=MESH_OT_hallr_centerline.bl_icon)
        layout.operator(MESH_OT_hallr_discretize.bl_idname, icon=MESH_OT_hallr_discretize.bl_icon)
        layout.separator()
        layout.operator(MESH_OT_hallr_select_vertices_until_intersection.bl_idname,
                        icon=MESH_OT_hallr_select_vertices_until_intersection.bl_icon)
        layout.operator(MESH_OT_hallr_select_intersection_vertices.bl_idname,
                        icon=MESH_OT_hallr_select_intersection_vertices.bl_icon)
        layout.operator(MESH_OT_hallr_select_end_vertices.bl_idname, icon=MESH_OT_hallr_select_end_vertices.bl_icon)
        layout.operator(MESH_OT_hallr_select_collinear_edges.bl_idname,
                        icon=MESH_OT_hallr_select_collinear_edges.bl_icon)
        layout.operator(MESH_OT_hallr_knife_intersect.bl_idname, icon=MESH_OT_hallr_knife_intersect.bl_icon)
        layout.separator()
        layout.operator(MESH_OT_hallr_random_vertices.bl_idname, icon=MESH_OT_hallr_random_vertices.bl_icon)
        layout.operator(MESH_OT_hallr_triangulate_and_flatten.bl_idname,
                        icon=MESH_OT_hallr_triangulate_and_flatten.bl_icon),
        layout.operator(MESH_OT_hallr_mesh_cleanup.bl_idname,
                        icon=MESH_OT_hallr_triangulate_and_flatten.bl_icon)


# draw function for integration in menus
def menu_func(self, context):
    self.layout.menu("VIEW3D_MT_edit_mesh_hallr_meshtools")
    self.layout.separator()


# define classes for registration
classes = (
    VIEW3D_MT_edit_mesh_hallr_meshtools,
    MESH_OT_hallr_triangulate_and_flatten,
    MESH_OT_hallr_select_end_vertices,
    MESH_OT_hallr_select_intersection_vertices,
    MESH_OT_hallr_select_vertices_until_intersection,
    MESH_OT_hallr_select_collinear_edges,
    MESH_OT_hallr_convex_hull_2d,
    MESH_OT_hallr_knife_intersect,
    MESH_OT_hallr_voroni_mesh,
    MESH_OT_hallr_voronoi_diagram,
    MESH_OT_hallr_sdf_mesh_25D,
    MESH_OT_hallr_sdf_mesh,
    MESH_OT_hallr_2d_outline,
    MESH_OT_hallr_simplify_rdp,
    MESH_OT_hallr_centerline,
    MESH_OT_hallr_discretize,
    MESH_OT_hallr_random_vertices,
    MESH_OT_hallr_meta_volume,
    MESH_OT_hallr_mesh_cleanup,
)


# registering and menu integration
def register():
    try:
        for cls in classes:
            bpy.utils.register_class(cls)
    except Exception as e:
        print(f"Failed to register operator: {e}")
        raise e
    bpy.types.VIEW3D_MT_edit_mesh_context_menu.prepend(menu_func)


# unregistering and removing menus
def unregister():
    for cls in reversed(classes):
        try:
            bpy.utils.unregister_class(cls)
        except (RuntimeError, NameError):
            pass
    bpy.types.VIEW3D_MT_edit_mesh_context_menu.remove(menu_func)
