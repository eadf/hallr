import bpy
from . import hallr_ffi_utils

bl_info = {
    "name": "Hallr Simplify RDP",
    "category": "Object",
    "location": "View3D > Tools",
    "description": "This module does something useful.",
    "author": "EAD",
    "version": (0, 1, 0),
    "blender": (3, 4, 1),
    "warning": "This executes rust code on your computer",
}


class OBJECT_OT_hallr_simplify_rdp(bpy.types.Operator):
    """Line Simplification using the RDP Algorithm"""
    bl_idname = "object.simplify_rdp"
    bl_label = "Hallr Simplify RDP"
    bl_options = {'REGISTER', 'UNDO'}

    epsilon_props: bpy.props.FloatProperty(name="Epsilon", default=0.1, min=0, description="Amount of simplification")

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "simplify_rdp", "epsilon": str(self.epsilon_props)}

        # Call the Rust function
        vertices, indices, config = hallr_ffi_utils.call_rust_direct(config, obj, expect_line_string=True)
        print(f"Received {config} as the result from Rust!")
        if config.get("ERROR"):
            self.report({'ERROR'}, "" + config.get("ERROR"))
            return {'CANCELLED'}
        elif config.get("mesh.format") == "line":
            hallr_ffi_utils.handle_sliding_line_mesh_modify_actice_object(vertices, indices)
        else:
            self.report({'ERROR'}, "Unknown mesh format:" + config.get("mesh.format", "None"))
            return {'CANCELLED'}

        return {'FINISHED'}

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "epsilon_props", text="Epsion")


def VIEW3D_MT_hallr_simplify_rdp_menu_item(self, context):
    self.layout.operator(OBJECT_OT_hallr_simplify_rdp.bl_idname, text="MeshMach simplify 2d")


def register():
    bpy.utils.register_class(OBJECT_OT_hallr_simplify_rdp)
    bpy.types.VIEW3D_MT_object_convert.append(VIEW3D_MT_hallr_simplify_rdp_menu_item)


def unregister():
    try:
        bpy.utils.unregister_class(OBJECT_OT_hallr_simplify_rdp)
    except (RuntimeError, NameError):
        pass
    bpy.types.VIEW3D_MT_object_convert.remove(VIEW3D_MT_hallr_simplify_rdp_menu_item)


if __name__ == "__main__":
    unregister()
    register()