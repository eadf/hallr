"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
import time
import math
import bmesh

DEV_MODE = False  # Set this to False for distribution
if DEV_MODE:
    import hallr_ffi_utils
    from hallr_mesh_operators.common import BaseOperatorMixin
else:
    from .. import hallr_ffi_utils
    from ..hallr_mesh_operators.common import BaseOperatorMixin


# Isotropic Remeshing mesh operator
class MESH_OT_hallr_isotropic_remesh(bpy.types.Operator, BaseOperatorMixin):
    bl_idname = "mesh.hallr_meshtools_isotropic_remesh"
    bl_label = "Isotropic Remesh"
    bl_icon = 'MOD_MESHDEFORM'
    bl_description = "Remesh the mesh isotropically using the crate 'remesh'"
    bl_options = {'REGISTER', 'UNDO'}

    DEFAULT_COPLANAR_ANGLE_THRESHOLD_RAD = math.radians(5.0)
    DEFAULT_CREASE_ANGLE_THRESHOLD_RAD = math.radians(160)
    DEFAULT_EDGE_FLIP_QUALITY_WEIGHT = 1.1
    DEFAULT_COLLAPSE_QEM_WEIGHT = 0.5
    DEFAULT_SMOOTH_WEIGHT = 10.0  # note: this is a percentage

    iterations_count_prop: bpy.props.IntProperty(
        name="Iterations",
        description="Number of iterations for remeshing. Increase this if your remeshed mesh contains irregularities."
                    "Higher values improve mesh quality but increase computation time.",
        default=10,
        min=1,
        max=100
    )

    target_edge_length_prop: bpy.props.FloatProperty(
        name="Target Edge Length",
        description="Target edge length after remeshing. Warning: Setting this too small will significantly increase processing time",
        default=DEFAULT_COLLAPSE_QEM_WEIGHT,
        min=0.001,
        max=3.0,
        precision=6,
        unit='LENGTH'
    )

    split_edges_prop: bpy.props.BoolProperty(
        name="Split Edges",
        description="Use edge splitting during remeshing",
        default=True
    )

    collapse_edges_prop: bpy.props.EnumProperty(
        name="Collapse Edges",
        description="Use edge collapsing during remeshing",
        items=[
            ('DISABLED', "Disabled", "Disable edge flipping during remeshing"),
            ('DIHEDRAL', "Dihedral angle", "Use dihedral angle priority during edge collapse (faster)"),
            ('QEM', "Qem", "Use a quadratic error measurements priority during edge collapse (slow)"),
        ],
        default='QEM'
    )

    collapse_qem_threshold_prop: bpy.props.FloatProperty(
        name="Quadratic error",
        description="The threshold used by QEM edge-collapse, as a percentage of target edge length",
        default=5.0,
        min=0.1,
        max=90,
        precision=3,
        subtype='PERCENTAGE'
    )

    flip_edges_prop: bpy.props.EnumProperty(
        name="Flip Edges",
        description="Use edge flipping method during remeshing",
        items=[
            ('DISABLED', "Disabled", "Disable edge flipping during remeshing"),
            ('VALENCE', "Valence", "Use valence-based priority during edge flipping"),
            ('QUALITY', "Quality", "Use a valence then aspect-ration priority during edge flipping"),
        ],
        default='QUALITY'
    )

    quality_threshold_use_default_prop: bpy.props.BoolProperty(
        name=f"Default threshold {DEFAULT_EDGE_FLIP_QUALITY_WEIGHT}",
        description="Use default quality threshold",
        default=True,
    )

    quality_threshold_prop: bpy.props.FloatProperty(
        name="Quality Weight",
        description="Threshold for aspect-ratio quality in edge flipping",
        default=DEFAULT_EDGE_FLIP_QUALITY_WEIGHT,
        min=1.0,
        max=1.3
    )

    smooth_vertices_prop: bpy.props.BoolProperty(
        name="🚧 Smooth Vertices 🚧",
        description="Allow vertex smoothing during remeshing. Work in progress",
        default=False
    )

    smooth_use_default_prop: bpy.props.BoolProperty(
        name="Use default smooth weight",
        description="Use default or custom smoothing weight",
        default=True
    )

    smooth_weight_value_prop: bpy.props.FloatProperty(
        name="Smooth weight",
        description="Smooth weight as a percentage of the target edge length",
        default=DEFAULT_SMOOTH_WEIGHT,
        min=1.0,
        max=50.0,
        subtype='PERCENTAGE',
    )

    coplanar_threshold_use_default_prop: bpy.props.BoolProperty(
        description="Use default or custom coplanarity threshold value. Dihedral angle between merge-able triangles",
        default=True
    )

    coplanar_threshold_value_prop: bpy.props.FloatProperty(
        name="Coplanarity Threshold",
        description="Dihedral angle between merge-able triangles",
        default=DEFAULT_COPLANAR_ANGLE_THRESHOLD_RAD,
        min=0.0,
        max=math.radians(90),
        subtype='ANGLE',
    )

    crease_threshold_use_default_prop: bpy.props.BoolProperty(
        description="Use default or custom sharp crease threshold value. The algorithm will not create sharper creases than this angle",
        default=True
    )
    crease_threshold_value_prop: bpy.props.FloatProperty(
        name="Sharp crease threshold",
        description="The algorithm will not create sharper creases than this angle",
        default=DEFAULT_CREASE_ANGLE_THRESHOLD_RAD,
        min=math.radians(100.0),
        max=math.radians(179.0),
        subtype='ANGLE',
    )

    deny_non_manifold_prop: bpy.props.BoolProperty(
        name="Deny non manifold mesh",
        description="Check if the mesh is non-manifold before sending to Rust functions",
        default=True
    )

    manifold_not_checked = True

    def invoke(self, context, event):
        self.manifold_not_checked = True
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def execute(self, context):
        wall_clock = time.perf_counter()
        obj = context.active_object

        if self.deny_non_manifold_prop:
            # Switch to edit mode and select non-manifold geometry
            if obj.mode != 'EDIT':
                bpy.ops.object.mode_set(mode='EDIT')
            original_select_mode = context.tool_settings.mesh_select_mode[:]
            bpy.ops.mesh.select_mode(type='VERT')
            bpy.ops.mesh.select_all(action='DESELECT')
            bpy.ops.mesh.select_non_manifold()

            manifold_check_time = time.perf_counter()

            # Get the selected elements count
            bm = bmesh.from_edit_mesh(obj.data)
            non_manifold_count = sum(1 for v in bm.verts if v.select)
            bm.free()
            self.manifold_not_checked = False

            # print(f"Python: manifold_check: {hallr_ffi_utils._duration_to_str(time.perf_counter() - manifold_check_time)}")

            if non_manifold_count > 0:
                self.report({'ERROR'},
                            f"Mesh is not manifold! Found {non_manifold_count} problem areas. Problem areas have been selected.")
                return {'CANCELLED'}
            else:
                context.tool_settings.mesh_select_mode = original_select_mode

        if self.coplanar_threshold_use_default_prop:
            coplanar_angle_rad = self.DEFAULT_COPLANAR_ANGLE_THRESHOLD_RAD
        else:
            coplanar_angle_rad = self.coplanar_threshold_value_prop
        if self.crease_threshold_use_default_prop:
            crease_angle_rad = self.DEFAULT_CREASE_ANGLE_THRESHOLD_RAD
        else:
            crease_angle_rad = self.crease_threshold_value_prop

        config = {
            hallr_ffi_utils.COMMAND_TAG: "isotropic_remesh",
            "ITERATIONS_COUNT": str(self.iterations_count_prop),
            "TARGET_EDGE_LENGTH": str(self.target_edge_length_prop),
            "SPLIT_EDGES": str(self.split_edges_prop),
            "COLLAPSE_EDGES": str(self.collapse_edges_prop),
            "FLIP_EDGES": str(self.flip_edges_prop),
            "COPLANAR_ANGLE_THRESHOLD": str(math.degrees(coplanar_angle_rad)),
            "CREASE_ANGLE_THRESHOLD": str(math.degrees(crease_angle_rad)),
        }

        if self.collapse_edges_prop == "QEM":
            config["COLLAPSE_QEM_THRESHOLD"] = str(self.collapse_qem_threshold_prop / 100.0)

        if self.smooth_vertices_prop:
            if self.smooth_use_default_prop:
                config["SMOOTH_WEIGHT"] = str(self.DEFAULT_SMOOTH_WEIGHT / 100.0)
            else:
                config["SMOOTH_WEIGHT"] = str(self.smooth_weight_value_prop / 100.0)

        if self.flip_edges_prop == 'QUALITY':
            if self.quality_threshold_use_default_prop:
                config["FLIP_QUALITY_THRESHOLD"] = str(self.DEFAULT_EDGE_FLIP_QUALITY_WEIGHT)
            else:
                config["FLIP_QUALITY_THRESHOLD"] = str(self.quality_threshold_prop)
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

        # print(f"Python: operation_execute: {hallr_ffi_utils._duration_to_str(time.perf_counter() - wall_clock)}")
        return {'FINISHED'}

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "iterations_count_prop")
        # Add target_edge_length with a warning message
        box = layout.box()
        row = box.row()
        row.prop(self, "target_edge_length_prop")
        # Add warning row with icon
        warning_row = box.row()
        warning_row.label(text="CAUTION: Small values will make the ", icon='ERROR')
        warning_row = box.row()
        warning_row.label(text="         operation take a long time, while ")
        warning_row = box.row()
        warning_row.label(text="         blender is unresponsive")
        warning_row.scale_y = 0.7
        layout.prop(self, "split_edges_prop")
        if self.collapse_edges_prop == "QEM":
            row = layout.row()
            row.prop(self, "collapse_edges_prop")
            row.prop(self, "collapse_qem_threshold_prop")
        else:
            layout.prop(self, "collapse_edges_prop")
        row = layout.row()
        if self.flip_edges_prop == 'QUALITY':
            row.prop(self, "flip_edges_prop", text='')
            if self.quality_threshold_use_default_prop:
                row.prop(self, "quality_threshold_use_default_prop")
            else:
                row.prop(self, "quality_threshold_use_default_prop", text='')
                row.prop(self, "quality_threshold_prop")
        else:
            row.prop(self, "flip_edges_prop")

        row = layout.row()
        if self.smooth_vertices_prop:
            sub = row.row()
            sub.alignment = 'LEFT'
            sub.prop(self, "smooth_vertices_prop", text='🚧')
            if self.smooth_use_default_prop:
                sub.prop(self, "smooth_use_default_prop")
            else:
                sub.prop(self, "smooth_use_default_prop", text='')
                sub.prop(self, "smooth_weight_value_prop")
        else:
            row.prop(self, "smooth_vertices_prop")

        row = layout.row()
        if self.coplanar_threshold_use_default_prop:
            row.prop(self, "coplanar_threshold_use_default_prop",
                     text=f"Default Coplanar threshold: {math.degrees(self.DEFAULT_COPLANAR_ANGLE_THRESHOLD_RAD)}°")
        else:
            split = row.split(factor=0.25)
            split.prop(self, "coplanar_threshold_use_default_prop", text='Default')
            split.prop(self, "coplanar_threshold_value_prop")

        row = layout.row()
        if self.crease_threshold_use_default_prop:
            row.prop(self, "crease_threshold_use_default_prop",
                     text=f"Default sharp crease threshold: {math.degrees(self.DEFAULT_CREASE_ANGLE_THRESHOLD_RAD)}°")
        else:
            split = row.split(factor=0.25)
            split.prop(self, "crease_threshold_use_default_prop", text='Default')
            split.prop(self, "crease_threshold_value_prop")

        # row = layout.row()
        # row.prop(self, "deny_non_manifold_prop")
