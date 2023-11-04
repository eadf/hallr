import bpy
from . import hallr_ffi_utils


class OBJECT_OT_hallr_simplify_rdp(bpy.types.Operator):
    """2D Line Simplification using the RDP Algorithm, works in the XY plane"""

    bl_idname = "object.hallr_simplify_rdp"
    bl_label = "Hallr 2D Simplify RDP"
    bl_options = {'REGISTER', 'UNDO'}

    epsilon_props: bpy.props.FloatProperty(name="Epsilon", default=0.1, min=0, description="Amount of simplification")
    simplify_3d_props: bpy.props.BoolProperty(
        name="Simplify 3d",
        description="When selected simplification will be done in 3d",
        default=True
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

        config = {"command": "simplify_rdp", "epsilon": str(self.epsilon_props),
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
        layout.prop(self, "epsilon_props")
        layout.prop(self, "simplify_3d_props")


def VIEW3D_MT_hallr_simplify_rdp_menu_item(self, context):
    self.layout.operator(OBJECT_OT_hallr_simplify_rdp.bl_idname)


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
