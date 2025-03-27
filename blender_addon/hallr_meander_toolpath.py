"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
import math
from . import hallr_ffi_utils

# Define the choices for the tool/probe property
probes_props_items = [
    ("BALL_NOSE", "Ball Nose", "Use a ball nose probe, a cylinder ending in a half-sphere"),
    ("SQUARE_END", "Square End", "Use a square end probe, just a cylinder"),
    ("TAPERED_END", "Tapered End", "Use a tapered end probe, radius is the largest radius and angle is the angle of "
                                   "the taper"),
]

# Define the choices for the search pattern property
patterns_props_items = [
    ("MEANDER", "Meander", "Meander scan pattern"),
    ("TRIANGULATION", "Triangulation", "2d Delaunay Triangulation")
]

# Define the choices for the search pattern property
bounding_props_items = [
    ("AABB", "Aabb", "Axis aligned bounding box"),
    ("CONVEX_HULL", "ConvexHull", "Convex hull bounds")
]


class HALLR_PT_MeanderToolpath(bpy.types.Panel):
    """A CNC toolpath/mesh probe operation, work in progress"""
    bl_label = "Meander Toolpath"
    bl_idname = "HALLR_PT_meander_toolpath"
    bl_space_type = "VIEW_3D"
    bl_region_type = "UI"
    bl_category = "Hallr tools"

    def draw(self, context):
        if context.mode != 'OBJECT':
            return

        layout = self.layout
        settings = context.scene.hallr_meander_settings

        row = layout.row(align=True)
        # Bounding shape selection
        if settings.bounding_shape is not None:
            row.operator("object.hallr_mt_select_bounding_shape", text="De-Select Bounding Shape", icon='X')
        else:
            row.operator("object.hallr_mt_select_bounding_shape", text="Select Bounding Shape", icon='EYEDROPPER')
        if settings.bounding_shape is not None:
            row.label(text=settings.bounding_shape.name, icon='CHECKMARK')

        if settings.bounding_shape is not None:
            layout.row(align=True).prop(settings, "bounds_props")

        row = layout.row(align=True)
        # 3D mesh for height offsets
        if settings.mesh is not None:
            row.operator("object.hallr_mt_select_mesh", text="De-Select mesh", icon='X')
        else:
            row.operator("object.hallr_mt_select_mesh", text="Select mesh", icon='EYEDROPPER')
        if settings.mesh is not None:
            row.label(text=settings.mesh.name, icon='CHECKMARK')

        layout.row(align=True).prop(settings, "enable_adaptive_scan_props")
        if settings.enable_adaptive_scan_props:
            layout.row(align=True).prop(settings, "z_jump_threshold_multiplier_props")
            layout.row(align=True).prop(settings, "xy_sample_dist_multiplier_props")
            layout.row(align=True).prop(settings, "enable_reduce_props")

        layout.row(align=True).prop(settings, "probe_props")
        if settings.probe_props == "TAPERED_END":
            layout.row(align=True).prop(settings, "probe_angle_props")

        layout.row(align=True).prop(settings, "probe_radius_props")
        layout.row(align=True).separator()
        layout.row(align=True).prop(settings, "step_props")
        layout.row(align=True).prop(settings, "minimum_z_props")
        layout.row(align=True).prop(settings, "pattern_props")

        # Generate tool-path button
        if (settings.bounding_shape is not None and
                settings.mesh is not None):
            layout.row(align=True).operator("object.hallr_mt_generate_mesh", text="Scan")


class OBJECT_OT_MT_SelectBoundingShape(bpy.types.Operator):
    """ Select the object that contains the bounding shape """
    bl_idname = "object.hallr_mt_select_bounding_shape"
    bl_label = "Select Bounding Shape"
    bl_description = (
        "Select the bounding shape"
    )

    def execute(self, context):
        # Check the bounding shape
        bounding_shape = bpy.context.active_object
        settings = context.scene.hallr_meander_settings

        if settings.bounding_shape is not None:
            settings.bounding_shape = None
            return {'FINISHED'}
        if bounding_shape.type != 'MESH':
            self.report({'ERROR'}, "The bounding shape should be of type 'MESH'.")
            settings.bounding_shape = None
            return {'CANCELLED'}
        # Ensure the bounding shape doesn't have any faces:
        if len(bounding_shape.data.polygons) > 0:
            self.report({'ERROR'}, "The bounding shape should not have faces. It should be a line object.")
            settings.bounding_shape = None
            return {'CANCELLED'}
        if not hallr_ffi_utils.is_loop(bounding_shape.data):
            self.report({'ERROR'}, "The bounding shape should be a continuous loop.")
            settings.bounding_shape = None
            return {'CANCELLED'}
        settings.bounding_shape = bounding_shape
        return {'FINISHED'}


class OBJECT_OT_MT_SelectMesh(bpy.types.Operator):
    """ Select the object that contains the mesh """
    bl_idname = "object.hallr_mt_select_mesh"
    bl_label = "Select Height Mesh"
    bl_description = (
        "Select the mesh object"
    )

    def execute(self, context):
        active_object = bpy.context.active_object
        settings = context.scene.hallr_meander_settings
        if settings.mesh is not None:
            settings.mesh = None
            return {'FINISHED'}

        if active_object.type != 'MESH':
            self.report({'ERROR'}, "The mesh shape should be of type 'MESH'.")
            settings.mesh = None
            return {'CANCELLED'}
        # Check if the mesh is triangulated
        for face in active_object.data.polygons:
            if len(face.vertices) != 3:
                self.report({'ERROR'}, "That mesh is not fully triangulated!.")
                settings.mesh = None
                return {'CANCELLED'}

        settings.mesh = active_object
        return {'FINISHED'}


class OBJECT_OT_MT_GenerateMesh(bpy.types.Operator):
    """ Execute the toolpath generation"""
    bl_idname = "object.hallr_mt_generate_mesh"
    bl_label = "Generate Toolpath (why are these read-only?)"

    # bl_options = {'REGISTER', 'UNDO'}

    def execute(self, context):
        # Check if all objects are selected
        settings = context.scene.hallr_meander_settings
        bounding_shape = settings.bounding_shape
        point_cloud = settings.mesh
        if (bounding_shape is not None and
                point_cloud is not None):
            # Print the names of the selected objects
            print("Bounding Shape:", bounding_shape.name)
            print("bounding type:", settings.bounds_props)
            print("Height Mesh:", point_cloud.name)

            config = {"probe_radius": str(settings.probe_radius_props),
                      "probe": str(settings.probe_props),
                      "bounds": str(settings.bounds_props),
                      "pattern": str(settings.pattern_props),
                      "step": str(settings.step_props),
                      "mesh.format": "triangulated",
                      "minimum_z": str(settings.minimum_z_props),
                      "command": "surface_scan"}
            if str(settings.probe_props) == "TAPERED_END":
                config["probe_angle"] = str(settings.probe_angle_props)

            if settings.enable_adaptive_scan_props:
                config["z_jump_threshold_multiplier"] = str(settings.z_jump_threshold_multiplier_props)
                config["xy_sample_dist_multiplier"] = str(settings.xy_sample_dist_multiplier_props)
                config["reduce_adaptive"] = str(settings.enable_reduce_props).lower()

            print("config:", config)
            # Call the Rust function
            vertices, indices, config = hallr_ffi_utils.call_rust(config, settings.mesh, settings.bounding_shape)

            # print(f"Received {config} as the result from Rust!")
            if config.get("ERROR"):
                self.report({'ERROR'}, "" + config.get("ERROR"))
                return {'CANCELLED'}
            # Check if the returned mesh format is triangulated
            if config.get("mesh.format") == "triangulated":
                hallr_ffi_utils.handle_triangle_mesh(vertices, indices)
            # Handle line format
            elif config.get("mesh.format") == "line":
                hallr_ffi_utils.handle_windows_line_new_object(vertices, indices)
            else:
                self.report({'ERROR'}, "Unknown mesh format:" + config.get("mesh.format", "None"))
                return {'CANCELLED'}
        return {'FINISHED'}

    def check(self, context):
        settings = context.scene.hallr_meander_settings
        return (settings.bounding_shape is not None and
                settings.mesh is not None
                )


# Property group to store selected objects
class MeanderToolpathSettings(bpy.types.PropertyGroup):
    bounding_shape: bpy.props.PointerProperty(type=bpy.types.Object)
    bounds_props: bpy.props.EnumProperty(
        name="Bounding box",
        description="Choose bounding box functionality",
        items=bounding_props_items,
        default="AABB"
    )
    mesh: bpy.props.PointerProperty(type=bpy.types.Object)
    enable_adaptive_scan_props: bpy.props.BoolProperty(
        name="Enable Adaptive Scan",
        description="Activates a more accurate scanning method, though it may result in longer processing times.",
    )
    enable_reduce_props: bpy.props.BoolProperty(
        name="Enable reduce for Adaptive Scan",
        description="Reduces collinear line sections to one single line",
    )
    z_jump_threshold_multiplier_props: bpy.props.FloatProperty(
        name="Z Jump Threshold Multiplier",
        description="Multiplier for step size to set max Z jump before adding a new sample.",
        default=0.5,
        min=0.05,
        max=1.0
    )
    xy_sample_dist_multiplier_props: bpy.props.FloatProperty(
        name="XY Sample Distance Multiplier",
        description="Multiplier of step size determining the minimum XY distance "
                    "between samples before stopping adaptive scanning.",
        default=0.5,
        min=0.05,
        max=1.0
    )
    probe_radius_props: bpy.props.FloatProperty(
        name="Tool Radius",
        description="Define the radius of the tool",
        default=0.5,
        min=0.01,
        max=10.0
    )
    step_props: bpy.props.FloatProperty(
        name="Step size",
        description="Define step size of the grid sampling",
        default=0.5,
        min=0.01,
        max=10.0,
    )
    minimum_z_props: bpy.props.FloatProperty(
        name="Minimum Z value",
        description="Define the minimum of reported Z value.",
        default=0.0,
        min=-100.0,
        max=100.0
    )
    probe_props: bpy.props.EnumProperty(
        name="Tool/Probe",
        description="Choose a tool or probe",
        items=probes_props_items,
        default="BALL_NOSE"
    )
    probe_angle_props: bpy.props.FloatProperty(
        name="Probe angle",
        description="Define the angle of the tapered probe (included angle).",
        default=math.radians(90.0),
        min=math.radians(50.0),
        max=math.radians(110.0),
        subtype='ANGLE',
    )
    pattern_props: bpy.props.EnumProperty(
        name="Scan Pattern",
        description="Choose a scan pattern",
        items=patterns_props_items,
        default="MEANDER",
    )


# Register classes and property group
classes = (
    MeanderToolpathSettings,
    HALLR_PT_MeanderToolpath,
    OBJECT_OT_MT_SelectBoundingShape,
    OBJECT_OT_MT_SelectMesh,
    OBJECT_OT_MT_GenerateMesh,
)


def register():
    for cls in classes:
        bpy.utils.register_class(cls)
    bpy.types.Scene.hallr_meander_settings = bpy.props.PointerProperty(type=MeanderToolpathSettings)


def unregister():
    for cls in reversed(classes):
        try:
            bpy.utils.unregister_class(cls)
        except (RuntimeError, NameError):
            pass
    try:
        del bpy.types.Scene.hallr_meander_settings
    except AttributeError:
        pass
