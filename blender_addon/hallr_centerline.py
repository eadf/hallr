"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
import math
from . import hallr_ffi_utils


class OBJECT_OT_hallr_centerline(bpy.types.Operator):
    """Finds the center line of closed geometry, works in the XY plane"""

    bl_idname = "object.hallr_centerline"
    bl_label = "Hallr 2D Centerline"
    bl_options = {'REGISTER', 'UNDO'}

    angle_props: bpy.props.FloatProperty(
        name="Angle",
        description="Edge rejection angle, edges with edge-to-segment angles larger than this will be rejected",
        default=math.radians(89.0),
        min=math.radians(0.000001),
        max=math.radians(89.999999),
        precision=6,
        subtype='ANGLE',
    )

    weld_props: bpy.props.BoolProperty(
        name="Weld the centerline to outline",
        description="Centerline and outline will share vertices if they intersect",
        default=True
    )

    keep_input_props: bpy.props.BoolProperty(
        name="Keep input edges",
        description="Will keep the input edges in the output, will override the weld property if inactive",
        default=True
    )

    negative_radius_props: bpy.props.BoolProperty(
        name="Negative radius",
        description="Represent voronoi edge distance to input geometry as a negative Z value",
        default=True
    )

    remove_internals_props: bpy.props.BoolProperty(
        name="Remove internal edges",
        description="Remove edges internal to islands for the geometry. I.e. it will remove geometry generated from "
                    "closed loops inside closed loops",
        default=True
    )

    distance_props: bpy.props.FloatProperty(
        name="Distance",
        description="Discrete distance as a percentage of the AABB. This value is used when sampling parabolic arc "
                    "edges. It is also used for RDP simplification.",
        default=0.005,
        min=0.001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    simplify_props: bpy.props.BoolProperty(
        name="Simplify line strings",
        description="Simplify voronoi edges connected as in a line string. The 'distance' property is used.",
        default=True
    )

    @classmethod
    def poll(cls, context):
        ob = context.active_object
        return ob and ob.type == 'MESH'

    def execute(self, context):
        obj = context.active_object

        if obj.type != 'MESH':
            self.report({'ERROR'}, "Active object is not a mesh!")
            return {'CANCELLED'}

        # Ensure the object is in object mode
        bpy.ops.object.mode_set(mode='OBJECT')

        config = {"command": "centerline",
                  "ANGLE": str(math.degrees(self.angle_props)),
                  "REMOVE_INTERNALS"
                  : str(self.remove_internals_props).lower(),
                  "KEEP_INPUT"
                  : str(self.keep_input_props).lower(),
                  "NEGATIVE_RADIUS"
                  : str(self.negative_radius_props).lower(),
                  "DISTANCE"
                  : str(self.distance_props),
                  "SIMPLIFY"
                  : str(self.simplify_props).lower(),
                  "WELD"
                  : str(self.weld_props).lower(),
                  }
        # Call the Rust function
        vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, obj, use_line_chunks=True)
        hallr_ffi_utils.handle_received_object_replace_active(obj, config_out, vertices, indices)

        return {'FINISHED'}

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "angle_props")
        if self.keep_input_props:
            layout.prop(self, "weld_props")
        layout.prop(self, "keep_input_props")
        layout.prop(self, "negative_radius_props")
        layout.prop(self, "remove_internals_props")
        if self.simplify_props:
            layout.prop(self, "distance_props")
        layout.prop(self, "simplify_props")


def VIEW3D_MT_hallr_centerline_menu_item(self, context):
    self.layout.operator(OBJECT_OT_hallr_centerline.bl_idname)


def register():
    bpy.utils.register_class(OBJECT_OT_hallr_centerline)
    bpy.types.VIEW3D_MT_object_convert.append(VIEW3D_MT_hallr_centerline_menu_item)


def unregister():
    try:
        bpy.utils.unregister_class(OBJECT_OT_hallr_centerline)
    except (RuntimeError, NameError):
        pass
    bpy.types.VIEW3D_MT_object_convert.remove(VIEW3D_MT_hallr_centerline_menu_item)


if __name__ == "__main__":
    unregister()
    register()
