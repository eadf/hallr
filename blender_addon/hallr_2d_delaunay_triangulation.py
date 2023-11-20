"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
from . import hallr_ffi_utils

# Define the choices for the search pattern property
bounding_props_items = [
    ("AABB", "Aabb", "Axis aligned bounding box"),
    ("CONVEX_HULL", "ConvexHull", "Convex hull bounds")
]


class HALLR_PT_DelaunayTriangulation2D(bpy.types.Panel):
    """2½D Delaunay Triangulation, will use the XY plane to stitch together point clouds"""
    bl_label = "Delaunay triangulation 2D"
    bl_space_type = "VIEW_3D"
    bl_region_type = "UI"
    bl_category = "Hallr tools"

    def draw(self, context):
        layout = self.layout

        # Create a row where the buttons are aligned to each other.
        # layout.label(text=" Aligned Row:")

        row = layout.row(align=True)
        # Bounding shape selection
        # row.label(text="Bounding Shape:")
        row.operator("object.hallr_dt2_select_bounding_shape", text="Select Bounding Shape")
        if context.scene.hallr_dt2_delaunay_settings.bounding_shape:
            row.label(text=context.scene.hallr_dt2_delaunay_settings.bounding_shape.name, icon='CHECKMARK')

        if context.scene.hallr_dt2_delaunay_settings.bounding_shape is not None:
            row = layout.row(align=True)
            row.prop(context.scene.hallr_dt2_delaunay_settings, "bounds_props")

        row = layout.row(align=True)
        # 3D mesh/point cloud for height offsets
        row.operator("object.hallr_dt2_select_point_cloud", text="Select Point cloud")
        if context.scene.hallr_dt2_delaunay_settings.point_cloud:
            row.label(text=context.scene.hallr_dt2_delaunay_settings.point_cloud.name, icon='CHECKMARK')

        # Generate toolpath button
        if (context.scene.hallr_dt2_delaunay_settings.bounding_shape is not None and
                context.scene.hallr_dt2_delaunay_settings.point_cloud is not None):
            layout.operator("object.hallr_d2t_generate_mesh", text="Generate Mesh")


class OBJECT_OT_SelectBoundingShape(bpy.types.Operator):
    bl_idname = "object.hallr_dt2_select_bounding_shape"
    bl_label = "Select Bounding Shape"

    def execute(self, context):
        # Check the bounding shape
        bounding_shape = bpy.context.active_object
        if bounding_shape.type != 'MESH':
            self.report({'ERROR'}, "The bounding shape should be of type 'MESH'.")
            context.scene.hallr_dt2_delaunay_settings.bounding_shape = None
            return {'CANCELLED'}
        # Ensure the bounding shape doesn't have any faces:
        if len(bounding_shape.data.polygons) > 0:
            self.report({'ERROR'}, "The bounding shape should not have faces. It should be a line object.")
            context.scene.hallr_dt2_delaunay_settings.bounding_shape = None
            return {'CANCELLED'}
        if not hallr_ffi_utils.is_loop(bounding_shape.data):
            self.report({'ERROR'}, "The bounding shape should be a continuous loop.")
            context.scene.hallr_dt2_delaunay_settings.bounding_shape = None
            return {'CANCELLED'}
        context.scene.hallr_dt2_delaunay_settings.bounding_shape = bounding_shape
        return {'FINISHED'}


class OBJECT_OT_SelectPointCloud(bpy.types.Operator):
    bl_idname = "object.hallr_dt2_select_point_cloud"
    bl_label = "Select Height Mesh"

    def execute(self, context):
        if bpy.context.active_object.type != 'MESH':
            self.report({'ERROR'}, "The bounding shape should be of type 'MESH'.")
            context.scene.hallr_dt2_delaunay_settings.point_cloud = None
            return {'CANCELLED'}
        context.scene.hallr_dt2_delaunay_settings.point_cloud = bpy.context.active_object
        return {'FINISHED'}


class OBJECT_OT_GenerateMesh(bpy.types.Operator):
    bl_idname = "object.hallr_d2t_generate_mesh"
    bl_label = "Generate Toolpath"

    def execute(self, context):
        # Check if all objects are selected
        bounding_shape = context.scene.hallr_dt2_delaunay_settings.bounding_shape
        point_cloud = context.scene.hallr_dt2_delaunay_settings.point_cloud
        bounds_props = context.scene.hallr_dt2_delaunay_settings.bounds_props
        if (bounding_shape is not None and
                point_cloud is not None):
            # Print the names of the selected objects
            print("Bounding Shape:", bounding_shape.name)
            print("Height Mesh:", point_cloud.name)
            print("bounding type:", bounds_props)

            config = {"bounds": str(bounds_props),
                      "mesh.format": "point_cloud",
                      "command": "2d_delaunay_triangulation"}
            # Call the Rust function
            vertices, indices, config = hallr_ffi_utils.call_rust(config, point_cloud, bounding_shape)

            print(f"Received {config} as the result from Rust!")
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
        return (context.scene.hallr_dt2_delaunay_settings.bounding_shape is not None and
                context.scene.hallr_dt2_delaunay_settings.point_cloud is not None
                )


# Property group to store selected objects
class DelaunaySettings(bpy.types.PropertyGroup):
    bounding_shape: bpy.props.PointerProperty(type=bpy.types.Object)
    bounds_props: bpy.props.EnumProperty(
        name="Bounding box",
        description="Choose a bounding box",
        items=bounding_props_items,
        default="AABB"
    )
    point_cloud: bpy.props.PointerProperty(type=bpy.types.Object)


# Register classes and property group
classes = (
    DelaunaySettings,
    HALLR_PT_DelaunayTriangulation2D,
    OBJECT_OT_SelectBoundingShape,
    OBJECT_OT_SelectPointCloud,
    OBJECT_OT_GenerateMesh,
)


def register():
    for cls in classes:
        bpy.utils.register_class(cls)
    bpy.types.Scene.hallr_dt2_delaunay_settings = bpy.props.PointerProperty(type=DelaunaySettings)


def unregister():
    for cls in reversed(classes):
        try:
            bpy.utils.unregister_class(cls)
        except (RuntimeError, NameError):
            pass
    try:
        del bpy.types.Scene.hallr_dt2_delaunay_settings
    except AttributeError:
        pass
