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


# SDF mesh 2½D operator
class MESH_OT_hallr_sdf_mesh_25d(bpy.types.Operator, BaseOperatorMixin):
    """Tooltip: Generate a 3D SDF mesh from 2½D edges."""
    bl_idname = "mesh.hallr_meshtools_sdf_mesh_2_5"
    bl_label = "SDF Mesh 2½D"
    bl_icon = "MESH_CONE"
    bl_description = (
        "Generate a 3D mesh from 2½D edges. Typically this operation works on the data generated from the centerline operation."
        "The geometry should placed on the XY plane intersecting the origin."
        "Each edge is converted into a SDF cone with its endpoint (X, Y) as the tip and Z.abs() as the radius."
        "The resulting mesh will preserve the 2D outline while inflating it based on the median-axis distance."
    )
    bl_options = {'REGISTER', 'UNDO'}

    sdf_divisions_prop: bpy.props.IntProperty(
        name="Voxel Divisions",
        description="The longest axis of the model will be divided into this number of voxels; the other axes "
                    "will have a proportionally equal number of voxels.",
        default=100,
        min=50,
        max=600,
        subtype='UNSIGNED'
    )

    sdf_radius_multiplier_prop: bpy.props.FloatProperty(
        name="Radius multiplier",
        description="Radius multiplier",
        default=1.0,
        min=0.01,
        max=5.0,
        precision=6,
    )

    backend_variant_items = (
        ("sdf_mesh_2½_fsn", "Fast Surface Nets", "use fast_surface_nets backend"),
        ("sdf_mesh_2½_saft", "Saft", "use saft backend"),
    )

    cmd_backend_prop: bpy.props.EnumProperty(name="Backend", items=backend_variant_items, default="sdf_mesh_2½_fsn")

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

        config = {hallr_ffi_utils.COMMAND_TAG: self.cmd_backend_prop,
                  "SDF_DIVISIONS": str(self.sdf_divisions_prop),
                  "SDF_RADIUS_MULTIPLIER": str(self.sdf_radius_multiplier_prop),
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
        row.label(icon='DRIVER_DISTANCE')
        row.prop(self, "sdf_divisions_prop")
        row = layout.row()
        row.prop(self, "sdf_radius_multiplier_prop")
        row = layout.row()
        row.prop(self, "cmd_backend_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop
