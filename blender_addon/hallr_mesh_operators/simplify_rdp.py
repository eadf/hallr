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


class MESH_OT_hallr_simplify_rdp(bpy.types.Operator, BaseOperatorMixin):
    """Line Simplification using the RDP Algorithm, for 2d and 3d lines"""

    bl_idname = "mesh.hallr_simplify_rdp"
    bl_icon = 'OUTLINER_DATA_CURVE'
    bl_label = "Hallr Simplify RDP"
    bl_options = {'REGISTER', 'UNDO'}

    simplify_3d_prop: bpy.props.BoolProperty(
        name="Simplify 3d",
        description="Simplification will be done in 3d if selected",
        default=True)

    simplify_distance_prop: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discrete distance as a percentage of the longest axis of the model. This value is used for RDP "
                    "simplification.",
        default=0.10,
        min=0.001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def execute(self, context):
        wall_clock = time.perf_counter()
        obj = context.active_object

        config = {hallr_ffi_utils.COMMAND_TAG: "simplify_rdp", "simplify_distance": str(self.simplify_distance_prop),
                  "simplify_3d": str(self.simplify_3d_prop).lower()}

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

    def draw(self, context):
        layout = self.layout
        row = layout.row()
        row.label(icon='FIXED_SIZE')
        row.prop(self, "simplify_distance_prop")
        layout.prop(self, "simplify_3d_prop")
