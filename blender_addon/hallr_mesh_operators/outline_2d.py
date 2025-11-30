"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
import time

DEV_MODE = False  # Set this to False for distribution
if DEV_MODE:
    import hallr_ffi_utils
    from hallr_mesh_operators.common import BaseOperatorMixin
else:
    from .. import hallr_ffi_utils
    from ..hallr_mesh_operators.common import BaseOperatorMixin


class MESH_OT_hallr_2d_outline(bpy.types.Operator, BaseOperatorMixin):
    """Generates the 2d outline from 2D mesh objects"""

    bl_idname = "mesh.hallr_2d_outline"
    bl_icon = "MOD_OUTLINE"
    bl_label = "[XY] Hallr 2D Outline"
    bl_description = ("Outline 2d geometry into a wire frame, the geometry *must* be flat (Z=0) and on the XY plane"
                      "Typically the kind of mesh you get when you convert a text to mesh.")
    bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        wall_clock = time.perf_counter()
        obj = context.active_object

        config = {hallr_ffi_utils.COMMAND_TAG: "2d_outline"}

        try:
            # Call the Rust function
            _, info = hallr_ffi_utils.process_single_mesh(wall_clock, config, obj,
                                                          mesh_format=hallr_ffi_utils.MeshFormat.TRIANGULATED,
                                                          create_new=False)
            self.report({'INFO'}, info)
        except Exception as e:
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        return {'FINISHED'}
