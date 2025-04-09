"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
import os
import bmesh
from . import hallr_ffi_utils
from hallr_ffi_utils import MeshFormat

import os

# Cache the boolean status of HALLR_ALLOW_NON_MANIFOLD (set or not)
_DENY_NON_MANIFOLD = os.getenv("HALLR_ALLOW_NON_MANIFOLD") is None


def is_mesh_non_manifold(obj):
    if obj.type != 'MESH':
        return False
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    is_manifold = all(edge.is_manifold for edge in bm.edges)
    bm.free()
    return not is_manifold


# Baby Shark Decimate mesh operator
class MESH_OT_baby_shark_decimate(bpy.types.Operator):
    bl_idname = "mesh.hallr_meshtools_bs_decimate"
    bl_label = "Baby Shark Decimate"
    bl_icon = 'MOD_DECIM'
    bl_description = "Simplify mesh using the Baby Shark edge decimation algorithm"
    bl_options = {'REGISTER', 'UNDO'}

    error_threshold: bpy.props.FloatProperty(
        name="Error Threshold",
        description="Maximum error threshold for edge decimation",
        default=0.0005,
        min=0.00001,
        max=1.0,
        precision=6,
        unit='LENGTH'
    )

    min_faces_count: bpy.props.IntProperty(
        name="Minimum Faces",
        description="Minimum number of faces to preserve during simplification",
        default=10000,
        min=1,
        max=1000000
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH' and context.mode == 'EDIT_MESH'

    def execute(self, context):
        obj = context.active_object

        # no need to check for non-manifold mesh more than once.
        if _DENY_NON_MANIFOLD and bpy.context.active_operator != self:
            # Switch to edit mode and select non-manifold geometry
            bpy.ops.object.mode_set(mode='EDIT')
            original_select_mode = context.tool_settings.mesh_select_mode[:]
            bpy.ops.mesh.select_mode(type='VERT')
            bpy.ops.mesh.select_all(action='DESELECT')
            bpy.ops.mesh.select_non_manifold()

            # Get the selected elements count
            bm = bmesh.from_edit_mesh(obj.data)
            non_manifold_count = sum(1 for v in bm.verts if v.select)
            bm.free()

            if non_manifold_count > 0:
                self.report({'ERROR'},
                            f"Mesh is not manifold! Found {non_manifold_count} problem areas. Problem areas have been selected.")
                return {'CANCELLED'}
            else:
                context.tool_settings.mesh_select_mode = original_select_mode

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {
            "command": "baby_shark_decimate",
            "ERROR_THRESHOLD": str(self.error_threshold),
            "MIN_FACES_COUNT": str(self.min_faces_count)
        }

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=MeshFormat.TRIANGULATED, create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "error_threshold")
        layout.prop(self, "min_faces_count")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


# Baby Shark Isotropic Remeshing mesh operator
class MESH_OT_baby_shark_isotropic_remesh(bpy.types.Operator):
    bl_idname = "mesh.hallr_meshtools_bs_isotropic_remesh"
    bl_label = "Baby Shark Isotropic Remesh"
    bl_icon = 'MOD_MESHDEFORM'
    bl_description = "Remesh the mesh isotropically using the Baby Shark algorithm"
    bl_options = {'REGISTER', 'UNDO'}

    iterations_count_prop: bpy.props.IntProperty(
        name="Iterations",
        description="Number of iterations for remeshing. Increase this if your remeshed mesh contains irregularities."
                    "Higher values improve mesh quality but increase computation time.",
        default=10,
        min=1,
        max=100
    )

    target_edge_length_prop: bpy.props.FloatProperty(
        name="Target Edge Length",
        description="Target edge length after remeshing. Warning: Setting this too small will significantly increase processing time",
        default=1.0,
        min=0.001,
        max=2.0,
        precision=6,
        unit='LENGTH'
    )

    split_edges_prop: bpy.props.BoolProperty(
        name="Split Edges",
        description="Allow edge splitting during remeshing",
        default=True
    )

    collapse_edges_prop: bpy.props.BoolProperty(
        name="Collapse Edges",
        description="Allow edge collapsing during remeshing",
        default=True
    )

    flip_edges_prop: bpy.props.BoolProperty(
        name="Flip Edges",
        description="Allow edge flipping during remeshing",
        default=True
    )

    shift_vertices_prop: bpy.props.BoolProperty(
        name="Shift Vertices",
        description="Allow vertex shifting during remeshing",
        default=True
    )

    project_vertices_prop: bpy.props.BoolProperty(
        name="Project Vertices",
        description="Project vertices back to the original surface",
        default=True
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH' and context.mode == 'EDIT_MESH'

    def execute(self, context):
        obj = context.active_object

        # no need to check for non-manifold mesh more than once.
        if _DENY_NON_MANIFOLD and bpy.context.active_operator != self:
            # Switch to edit mode and select non-manifold geometry
            bpy.ops.object.mode_set(mode='EDIT')
            original_select_mode = context.tool_settings.mesh_select_mode[:]
            bpy.ops.mesh.select_mode(type='VERT')
            bpy.ops.mesh.select_all(action='DESELECT')
            bpy.ops.mesh.select_non_manifold()

            # Get the selected elements count
            bm = bmesh.from_edit_mesh(obj.data)
            non_manifold_count = sum(1 for v in bm.verts if v.select)
            bm.free()

            if non_manifold_count > 0:
                self.report({'ERROR'},
                            f"Mesh is not manifold! Found {non_manifold_count} problem areas. Problem areas have been selected.")
                return {'CANCELLED'}
            else:
                context.tool_settings.mesh_select_mode = original_select_mode

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {
            "command": "baby_shark_isotropic_remesh",
            "ITERATIONS_COUNT": str(self.iterations_count_prop),
            "TARGET_EDGE_LENGTH": str(self.target_edge_length_prop),
            "SPLIT_EDGES": str(self.split_edges_prop),
            "COLLAPSE_EDGES": str(self.collapse_edges_prop),
            "FLIP_EDGES": str(self.flip_edges_prop),
            "SHIFT_VERTICES": str(self.shift_vertices_prop),
            "PROJECT_VERTICES": str(self.project_vertices_prop)
        }

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=MeshFormat.TRIANGULATED, create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "iterations_count_prop")
        # Add target_edge_length with a warning message
        box = layout.box()
        row = box.row()
        row.prop(self, "target_edge_length_prop")
        # Add warning row with icon
        warning_row = box.row()
        warning_row.label(text="CAUTION: Small values will make the ", icon='ERROR')
        warning_row = box.row()
        warning_row.label(text="         operation take a long time, while ")
        warning_row = box.row()
        warning_row.label(text="         blender is unresponsive")
        warning_row.scale_y = 0.7
        layout.prop(self, "split_edges_prop")
        layout.prop(self, "collapse_edges_prop")
        layout.prop(self, "flip_edges_prop")
        layout.prop(self, "shift_vertices_prop")
        layout.prop(self, "project_vertices_prop")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


# Mesh Offset Operator
class MESH_OT_baby_shark_mesh_offset(bpy.types.Operator):
    bl_idname = "mesh.hallr_meshtools_bs_offset"
    bl_label = "Baby Shark Mesh Offset"
    bl_icon = 'MOD_SKIN'
    bl_description = "Offset mesh surface by converting to volume and back"
    bl_options = {'REGISTER', 'UNDO'}

    voxel_size_prop: bpy.props.FloatProperty(
        name="Voxel Size",
        description="Size of voxels for volume conversion",
        default=0.2,
        min=0.01,
        max=5.0,
        precision=3,
        unit='LENGTH'
    )

    offset_by_prop: bpy.props.FloatProperty(
        name="Offset Amount",
        description="Distance to offset the mesh surface",
        default=1.5,
        min=0.0,
        max=10.0,
        precision=2,
        unit='LENGTH'
    )

    remove_doubles_threshold_prop: bpy.props.FloatProperty(
        name="Merge Distance",
        description="Maximum distance between vertices to be merged (uses Blender's 'Remove Doubles' operation)",
        default=0.0001,
        min=0.000001,
        max=0.1,
        precision=6,
        unit='LENGTH',
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH' and context.mode == 'EDIT_MESH'

    def execute(self, context):
        obj = context.active_object

        # Ensure we're in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {
            "command": "baby_shark_mesh_offset",
            "VOXEL_SIZE": str(self.voxel_size_prop),
            "OFFSET_BY": str(self.offset_by_prop),
            "REMOVE_DOUBLES_THRESHOLD": str(self.remove_doubles_threshold_prop),
        }

        try:
            # Call the Rust function
            hallr_ffi_utils.process_single_mesh(config, obj, mesh_format=MeshFormat.TRIANGULATED, create_new=False)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.label(icon='MESH_CUBE')
        row.prop(self, "voxel_size_prop")
        layout.prop(self, "offset_by_prop")
        row = layout.row()
        row.label(icon='SNAP_MIDPOINT')
        row.prop(self, "remove_doubles_threshold_prop")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


class OBJECT_OT_baby_shark_boolean(bpy.types.Operator):
    """Custom Boolean Operation"""
    bl_idname = "object.custom_boolean"
    bl_label = "Baby Shark Boolean operation"
    bl_icon = 'MOD_BOOLEAN'
    bl_options = {'REGISTER', 'UNDO'}

    operation_prop: bpy.props.EnumProperty(
        name="Operation",
        items=[
            ('UNION', "Union", "Combine objects"),
            ('DIFFERENCE', "Difference", "Subtract from active object"),
            ('INTERSECT', "Intersect", "Keep overlapping parts"),
        ],
        default='DIFFERENCE'
    )

    voxel_size_prop: bpy.props.FloatProperty(
        name="Voxel Size",
        description="Size of voxels for volume conversion",
        default=0.2,
        min=0.01,
        max=5.0,
        precision=3,
        unit='LENGTH'
    )

    swap_operands_prop: bpy.props.BoolProperty(
        name="Swap Operands",
        description="Reverse the operation order, only meaningful for 'Difference'",
        default=False
    )

    remove_doubles_threshold_prop: bpy.props.FloatProperty(
        name="Merge Distance",
        description="Maximum distance between vertices to be merged (uses Blender's 'Remove Doubles' operation)",
        default=0.0001,
        min=0.000001,
        max=0.1,
        precision=6,
        unit='LENGTH'
    )

    @classmethod
    def poll(cls, context):
        return (context.mode == 'OBJECT' and
                len(context.selected_objects) == 2 and
                all(obj.type == 'MESH' for obj in context.selected_objects) and
                context.active_object is not None)

    def execute(self, context):
        if len(context.selected_objects) == 2:
            mesh_1 = context.selected_objects[0]
            mesh_2 = context.selected_objects[1]

            if _DENY_NON_MANIFOLD and bpy.context.active_operator != self and is_mesh_non_manifold(mesh_1):
                self.report(
                    {'ERROR'},
                    f"Object '{mesh_1.name}' is non-manifold. Fix it before proceeding."
                )
                return {'CANCELLED'}

            if _DENY_NON_MANIFOLD and bpy.context.active_operator != self and is_mesh_non_manifold(mesh_2):
                self.report(
                    {'ERROR'},
                    f"Object '{mesh_2.name}' is non-manifold. Fix it before proceeding."
                )
                return {'CANCELLED'}

            config = {"operation": str(self.operation_prop),
                      "swap": str(self.swap_operands_prop),
                      "voxel_size": str(self.voxel_size_prop),
                      "REMOVE_DOUBLES_THRESHOLD": str(self.remove_doubles_threshold_prop),
                      "command": "baby_shark_boolean"}

            try:
                # Call the Rust function
                hallr_ffi_utils.process_mesh_with_rust(config, primary_mesh=mesh_1,
                                                       secondary_mesh=mesh_2,
                                                       primary_format=MeshFormat.TRIANGULATED,
                                                       secondary_format=MeshFormat.TRIANGULATED,
                                                       create_new=True)
            except Exception as e:
                import traceback
                traceback.print_exc()
                self.report({'ERROR'}, f"Error: {e}")
                return {'CANCELLED'}

        return {'FINISHED'}

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self, width=300)

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "operation_prop")
        row = layout.row()
        row.label(icon='MESH_CUBE')
        row.prop(self, "voxel_size_prop")
        layout.prop(self, "swap_operands_prop")
        row = layout.row()
        row.label(icon='SNAP_MIDPOINT')
        row.prop(self, "remove_doubles_threshold_prop")


# Panel for redo-last (appears in F9 and Adjust Last Operation)
def draw_panel(self, context):
    layout = self.layout
    op = context.active_operator
    if op and op.bl_idname == "object.smart_boolean":
        layout.prop(op, "operation")


# menu containing all edit tools
class VIEW3D_MT_edit_mesh_baby_shark_operations(bpy.types.Menu):
    bl_label = "Baby Shark mesh operations"

    def draw(self, context):
        layout = self.layout
        layout.operator(MESH_OT_baby_shark_decimate.bl_idname, icon=MESH_OT_baby_shark_decimate.bl_icon)
        layout.operator(MESH_OT_baby_shark_isotropic_remesh.bl_idname, icon=MESH_OT_baby_shark_isotropic_remesh.bl_icon)
        layout.operator(MESH_OT_baby_shark_mesh_offset.bl_idname, icon=MESH_OT_baby_shark_mesh_offset.bl_icon)


# menu containing all object tools
class VIEW3D_MT_object_mesh_baby_shark_operations(bpy.types.Menu):
    bl_label = "Baby Shark mesh operations"

    def draw(self, context):
        layout = self.layout
        layout.separator()
        layout.operator_context = 'INVOKE_DEFAULT'
        layout.operator(OBJECT_OT_baby_shark_boolean.bl_idname, icon=OBJECT_OT_baby_shark_boolean.bl_icon)


# draw function for integration in menus
def edit_mode_menu_func(self, context):
    self.layout.menu("VIEW3D_MT_edit_mesh_baby_shark_operations")
    self.layout.separator()


def object_mode_menu_func(self, context):
    self.layout.menu("VIEW3D_MT_object_mesh_baby_shark_operations")
    self.layout.separator()


# registering and menu integration
def register():
    try:
        for cls in classes:
            bpy.utils.register_class(cls)
    except Exception as e:
        print(f"Failed to register operator: {e}")
        raise e
    bpy.types.VIEW3D_MT_object_context_menu.prepend(object_mode_menu_func)
    bpy.types.VIEW3D_MT_edit_mesh_context_menu.prepend(edit_mode_menu_func)


# unregistering and removing menus
def unregister():
    for cls in reversed(classes):
        try:
            bpy.utils.unregister_class(cls)
        except (RuntimeError, NameError):
            pass
    bpy.types.VIEW3D_MT_object_context_menu.remove(object_mode_menu_func)
    bpy.types.VIEW3D_MT_edit_mesh_context_menu.remove(edit_mode_menu_func)


# define classes for registration
classes = (
    VIEW3D_MT_edit_mesh_baby_shark_operations,
    VIEW3D_MT_object_mesh_baby_shark_operations,
    OBJECT_OT_baby_shark_boolean,
    MESH_OT_baby_shark_isotropic_remesh,
    MESH_OT_baby_shark_decimate,
    MESH_OT_baby_shark_mesh_offset,
)
