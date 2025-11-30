"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
import time
import math

DEV_MODE = False  # Set this to False for distribution
if DEV_MODE:
    import hallr_ffi_utils
    from hallr_mesh_operators.common import BaseOperatorMixin
else:
    from .. import hallr_ffi_utils
    from ..hallr_mesh_operators.common import BaseOperatorMixin


class MESH_OT_hallr_centerline(bpy.types.Operator, BaseOperatorMixin):
    """Finds the center line of closed geometry, works in the XY plane"""

    bl_idname = "mesh.hallr_centerline"
    bl_icon = "CONE"
    bl_label = "[XY] Hallr 2D Centerline"
    bl_options = {'REGISTER', 'UNDO'}

    angle_prop: bpy.props.FloatProperty(
        name="Angle",
        description="Edge rejection angle, edges with edge-to-segment angles larger than this will be rejected",
        default=math.radians(89.0),
        min=math.radians(0.000001),
        max=math.radians(89.999999),
        precision=6,
        subtype='ANGLE',
    )

    keep_input_prop: bpy.props.BoolProperty(
        name="Keep input edges",
        description="Will keep the input edges in the output",
        default=True
    )

    negative_radius_prop: bpy.props.BoolProperty(
        name="Negative radius",
        description="Represent voronoi edge distance to input geometry as a negative Z value",
        default=True
    )

    remove_internals_prop: bpy.props.BoolProperty(
        name="Remove internal edges",
        description="Remove edges internal to islands for the geometry. I.e. it will remove geometry generated from "
                    "closed loops inside closed loops",
        default=True
    )

    distance_prop: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discrete distance as a percentage of the AABB. This value is used when sampling parabolic arc "
                    "edges. It is also used for RDP simplification.",
        default=0.05,
        min=0.001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    simplify_prop: bpy.props.BoolProperty(
        name="Simplify line strings",
        description="RDP Simplify voronoi edges connected as in a line string. The 'distance' property is used.",
        default=True
    )

    remove_doubles_threshold_prop: bpy.props.FloatProperty(
        name="Merge Distance",
        description="Maximum distance between vertices to be merged",
        default=0.001,
        min=0.000001,
        max=0.01,
        precision=6,
        unit='LENGTH'
    )

    use_remove_doubles_prop: bpy.props.BoolProperty(
        name="Use remove doubled",
        description="Activates the remove doubles feature",
        default=True
    )

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def execute(self, context):
        wall_clock = time.perf_counter()
        obj = context.active_object

        config = {hallr_ffi_utils.COMMAND_TAG: "centerline",
                  "ANGLE": str(math.degrees(self.angle_prop)),
                  "REMOVE_INTERNALS"
                  : str(self.remove_internals_prop).lower(),
                  "KEEP_INPUT"
                  : str(self.keep_input_prop).lower(),
                  "NEGATIVE_RADIUS"
                  : str(self.negative_radius_prop).lower(),
                  "DISTANCE"
                  : str(self.distance_prop),
                  "SIMPLIFY"
                  : str(self.simplify_prop).lower(),
                  }
        if self.use_remove_doubles_prop:
            config[hallr_ffi_utils.VERTEX_MERGE_TAG] = str(self.remove_doubles_threshold_prop)
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
        row.label(icon='FILTER')
        row.prop(self, "angle_prop")
        layout.prop(self, "keep_input_prop")
        layout.prop(self, "negative_radius_prop")
        layout.prop(self, "remove_internals_prop")
        row = layout.row()
        row.label(icon='FIXED_SIZE')
        row.prop(self, "distance_prop")
        layout.prop(self, "simplify_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop
