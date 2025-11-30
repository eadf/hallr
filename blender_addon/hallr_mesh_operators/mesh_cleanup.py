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
        wall_clock = time.perf_counter()
        obj = context.active_object

        config = {
            hallr_ffi_utils.COMMAND_TAG: "mesh_cleanup",
            "max_iterations": str(self.iterations_count_prop),
        }

        try:
            # Call the Rust function
            _, info = hallr_ffi_utils.process_single_mesh(wall_clock, config, obj,
                                                          mesh_format=hallr_ffi_utils.MeshFormat.TRIANGULATED,
                                                          create_new=False)
            self.report({'INFO'}, info)
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