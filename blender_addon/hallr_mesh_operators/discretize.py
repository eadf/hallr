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


# Discretize operator
class MESH_OT_hallr_discretize(bpy.types.Operator, BaseOperatorMixin):
    """Subdivide edges by length"""
    bl_idname = "mesh.hallr_meshtools_discretize"
    bl_label = "Subdivide by length"
    bl_icon = "CENTER_ONLY"
    bl_description = (
        "Subdivides edges by length."
    )
    bl_options = {'REGISTER', 'UNDO'}

    discretize_length_prop: bpy.props.FloatProperty(
        name="Length",
        description="Discretize length as a percentage of the total AABB. The edges will be split up by up to this "
                    "length, and no more",
        default=25.0,
        min=0.1,
        max=51,
        precision=3,
        subtype='PERCENTAGE'
    )

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def execute(self, context):
        wall_clock = time.perf_counter()
        obj = context.active_object

        config = {hallr_ffi_utils.COMMAND_TAG: "discretize",
                  "discretize_length": str(self.discretize_length_prop),
                  }

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
        row.prop(self, "discretize_length_prop")
