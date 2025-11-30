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
        wall_clock = time.perf_counter()
        obj = context.active_object

        bpy.context.view_layer.update()

        config = {hallr_ffi_utils.COMMAND_TAG: "knife_intersect"}

        try:
            # Call the Rust function
            _, info = hallr_ffi_utils.process_single_mesh(wall_clock, config, obj,
                                                          mesh_format=hallr_ffi_utils.MeshFormat.EDGES,
                                                          create_new=False)
            self.report({'INFO'}, info)
        except Exception as e:
            import traceback
            traceback.print_exc()
            self.report({'ERROR'}, f"Error: {e}")
            return {'CANCELLED'}

        return {'FINISHED'}
