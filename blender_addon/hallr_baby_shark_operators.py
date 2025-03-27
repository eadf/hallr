"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
import os
import bmesh
from . import hallr_ffi_utils

import os

# Cache the boolean status of HALLR_ALLOW_NON_MANIFOLD (set or not)
_DENY_NON_MANIFOLD = os.getenv("HALLR_ALLOW_NON_MANIFOLD") is None


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
        precision=6
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

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

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

        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

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

    iterations_count: bpy.props.IntProperty(
        name="Iterations",
        description="Number of iterations for remeshing",
        default=10,
        min=1,
        max=100
    )

    target_edge_length: bpy.props.FloatProperty(
        name="Target Edge Length",
        description="Target edge length after remeshing. Warning: Setting this too small will significantly increase processing time",
        default=1.0,
        min=0.001,
        max=2.0,
        precision=6
    )

    split_edges: bpy.props.BoolProperty(
        name="Split Edges",
        description="Allow edge splitting during remeshing",
        default=True
    )

    collapse_edges: bpy.props.BoolProperty(
        name="Collapse Edges",
        description="Allow edge collapsing during remeshing",
        default=True
    )

    flip_edges: bpy.props.BoolProperty(
        name="Flip Edges",
        description="Allow edge flipping during remeshing",
        default=True
    )

    shift_vertices: bpy.props.BoolProperty(
        name="Shift Vertices",
        description="Allow vertex shifting during remeshing",
        default=True
    )

    project_vertices: bpy.props.BoolProperty(
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

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

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
            "ITERATIONS_COUNT": str(self.iterations_count),
            "TARGET_EDGE_LENGTH": str(self.target_edge_length),
            "SPLIT_EDGES": str(self.split_edges),
            "COLLAPSE_EDGES": str(self.collapse_edges),
            "FLIP_EDGES": str(self.flip_edges),
            "SHIFT_VERTICES": str(self.shift_vertices),
            "PROJECT_VERTICES": str(self.project_vertices)
        }

        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "iterations_count")
        # Add target_edge_length with a warning message
        box = layout.box()
        row = box.row()
        row.prop(self, "target_edge_length")
        # Add warning row with icon
        warning_row = box.row()
        warning_row.label(text="CAUTION: Small values will make the ", icon='ERROR')
        warning_row = box.row()
        warning_row.label(text="         operation take a long time, while ")
        warning_row = box.row()
        warning_row.label(text="         blender is unresponsive")
        warning_row.scale_y = 0.7
        layout.prop(self, "split_edges")
        layout.prop(self, "collapse_edges")
        layout.prop(self, "flip_edges")
        layout.prop(self, "shift_vertices")
        layout.prop(self, "project_vertices")

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

    voxel_size: bpy.props.FloatProperty(
        name="Voxel Size",
        description="Size of voxels for volume conversion",
        default=0.2,
        min=0.01,
        max=5.0,
        precision=3
    )

    offset_by: bpy.props.FloatProperty(
        name="Offset Amount",
        description="Distance to offset the mesh surface",
        default=1.5,
        min=0.01,
        max=10.0,
        precision=2
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

        # Ensure we're in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {
            "command": "baby_shark_mesh_offset",
            "VOXEL_SIZE": str(self.voxel_size),
            "OFFSET_BY": str(self.offset_by)
        }

        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "voxel_size")
        layout.prop(self, "offset_by")

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)


class OBJECT_OT_baby_shark_boolean(bpy.types.Operator):
    """Custom Boolean Operation"""
    bl_idname = "object.custom_boolean"
    bl_label = "Boolean"
    bl_icon = 'MOD_BOOLEAN'
    bl_options = {'REGISTER', 'UNDO'}

    operation: bpy.props.EnumProperty(
        name="Operation",
        items=[
            ('UNION', "Union", "Combine objects"),
            ('DIFFERENCE', "Difference", "Subtract from active object"),
            ('INTERSECT', "Intersect", "Keep overlapping parts"),
        ],
        default='DIFFERENCE'
    )

    swap_operands: bpy.props.BoolProperty(
        name="Swap Operands",
        description="Reverse the operation order",
        default=False
    )

    apply_modifier: bpy.props.BoolProperty(
        name="Apply Immediately",
        description="Apply the boolean modifier right away",
        default=False
    )

    @classmethod
    def poll(cls, context):
        return (context.mode == 'OBJECT' and
                len(context.selected_objects) >= 2 and
                context.active_object is not None)

    def execute(self, context):
        # Your boolean operation implementation would go here
        # For now just print the settings
        print(f"Boolean operation: {self.operation}")
        print(f"Swap operands: {self.swap_operands}")
        print(f"Apply immediately: {self.apply_modifier}")

        self.report({'INFO'}, f"Custom boolean {self.operation.lower()} performed")
        return {'FINISHED'}

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self, width=300)


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
