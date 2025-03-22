import bpy
import bmesh
from . import hallr_ffi_utils


# menu containing all tools
class VIEW3D_MT_edit_mesh_hallr_bs_operations(bpy.types.Menu):
    bl_label = "Baby Shark mesh operations"

    def draw(self, context):
        layout = self.layout
        layout.operator("mesh.hallr_meshtools_bs_decimate")


# Baby Shark Simplify mesh operator
class Hallr_BS_Decimate(bpy.types.Operator):
    bl_idname = "mesh.hallr_meshtools_bs_decimate"
    bl_label = "Baby Shark Decimate"
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


# draw function for integration in menus
def menu_func(self, context):
    self.layout.menu("VIEW3D_MT_edit_mesh_hallr_bs_operations")
    self.layout.separator()


# define classes for registration
classes = (
    VIEW3D_MT_edit_mesh_hallr_bs_operations,
    Hallr_BS_Decimate,
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
