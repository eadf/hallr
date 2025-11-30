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


# SDF mesh operator
class MESH_OT_hallr_sdf_mesh(bpy.types.Operator, BaseOperatorMixin):
    """Generate a 3D SDF mesh from 3d edges."""
    bl_idname = "mesh.hallr_meshtools_sdf_mesh"
    bl_label = "SDF Mesh"
    bl_icon = "MESH_ICOSPHERE"
    bl_description = (
        "Generate a 3D mesh from 3D edges."
        "Each edge is converted into a SDF tube with a predefined radius."
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

    sdf_radius_prop: bpy.props.FloatProperty(
        name="Radius",
        description="Voxel tube radius as a percentage of the total AABB",
        default=1.0,
        min=0.01,
        max=19.9999,
        precision=6,
        subtype='PERCENTAGE'
    )

    backend_variant_items = (
        ("sdf_mesh", "Fast Surface Nets", "use fast_surface_nets backend"),
        ("sdf_mesh_saft", "Saft", "use saft backend"),
    )
    cmd_backend_prop: bpy.props.EnumProperty(name="Backend", items=backend_variant_items, default="sdf_mesh")

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
                  "SDF_RADIUS_MULTIPLIER": str(self.sdf_radius_prop),
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
        row.prop(self, "sdf_radius_prop")
        row = layout.row()
        row.prop(self, "cmd_backend_prop")
        row = layout.row()
        row.prop(self, "use_remove_doubles_prop", text="")
        right_side = row.split(factor=0.99)
        icon_area = right_side.row(align=True)
        icon_area.label(text="", icon='SNAP_MIDPOINT')
        icon_area.prop(self, "remove_doubles_threshold_prop")
        icon_area.enabled = self.use_remove_doubles_prop
