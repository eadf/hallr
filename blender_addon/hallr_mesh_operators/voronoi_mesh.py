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


# Voronoi mesh operator
class MESH_OT_hallr_voronoi_mesh(bpy.types.Operator, BaseOperatorMixin):
    bl_idname = "mesh.hallr_meshtools_voronoi_mesh"
    bl_label = "[XY] Voronoi Mesh"
    bl_icon = "MESH_UVSPHERE"
    bl_description = ("Calculate voronoi diagram and add mesh, the geometry must be flat and on a plane intersecting "
                      "origin. It also must be encircled by an outer continuous loop")
    bl_options = {'REGISTER', 'UNDO'}

    distance_prop: bpy.props.FloatProperty(
        name="Discretization distance",
        description="Discretization distance as a percentage of the total AABB length. This value is used when sampling"
                    "parabolic arc edges. Smaller value gives a finer step distance.",
        default=0.1,
        min=0.0001,
        max=4.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    negative_radius_prop: bpy.props.BoolProperty(
        name="Negative radius",
        description="Represent voronoi edge distance to input geometry as a negative Z value",
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

        config = {hallr_ffi_utils.COMMAND_TAG: "voronoi_mesh",
                  "DISTANCE": str(self.distance_prop),
                  "NEGATIVE_RADIUS": str(self.negative_radius_prop).lower(),
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
        row.label(icon='FIXED_SIZE')
        row.prop(self, "distance_prop")
        layout.prop(self, "negative_radius_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop
