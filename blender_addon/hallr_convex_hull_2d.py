import bpy
from . import hallr_ffi_utils


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
        vertices, indices, config = hallr_ffi_utils.call_rust(config, active_object, only_selected_vertices=True)

        print(f"Received {config} as the result from Rust!")
        if config.get("ERROR"):
            self.report({'ERROR'}, "" + config.get("ERROR"))
            return {'CANCELLED'}
        # Check if the returned mesh format is triangulated
        if config.get("mesh.format") == "triangulated":
            hallr_ffi_utils.handle_triangle_mesh(vertices, indices)
        # Handle line format
        elif config.get("mesh.format") == "line":
            hallr_ffi_utils.handle_windows_line_new_object(vertices, indices)
        else:
            self.report({'ERROR'}, "Unknown mesh format:" + config.get("mesh.format", "None"))
            return {'CANCELLED'}

        # Switch back to edit mode
        bpy.context.view_layer.objects.active = active_object
        bpy.ops.object.mode_set(mode='EDIT')

        return {'FINISHED'}


def VIEW3D_MT_hallr_convex_hull_2d_menu_item(self, context):
    self.layout.operator(MESH_OT_hallr_convex_hull_2d.bl_idname)


def register():
    bpy.utils.register_class(MESH_OT_hallr_convex_hull_2d)
    bpy.types.VIEW3D_MT_edit_mesh.append(VIEW3D_MT_hallr_convex_hull_2d_menu_item)


def unregister():
    try:
        bpy.utils.unregister_class(MESH_OT_hallr_convex_hull_2d)
    except (RuntimeError, NameError):
        pass
    # if hasattr(bpy.types, 'YOUR_OT_convex_hull_2d'):
    #    bpy.utils.unregister_class(YOUR_OT_convex_hull_2d)
    bpy.types.VIEW3D_MT_edit_mesh.remove(VIEW3D_MT_hallr_convex_hull_2d_menu_item)


if __name__ == "__main__":
    register()
