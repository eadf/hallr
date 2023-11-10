"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
from . import hallr_ffi_utils


class OBJECT_OT_hallr_simplify_rdp(bpy.types.Operator):
    """2D Line Simplification using the RDP Algorithm, works in the XY plane"""

    bl_idname = "object.hallr_2d_outline"
    bl_label = "Hallr 2D Outline"
    bl_description = ("Outline 2d geometry into a wire frame, the geometry must be flat and on a plane intersecting "
                      "origin")
    bl_options = {'REGISTER', 'UNDO'}

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

        config = {"command": "2d_outline"}

        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=False)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}


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
