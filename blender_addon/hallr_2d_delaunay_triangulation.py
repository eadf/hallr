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
    bl_idname = "HALLR_PT_delaunay_triangulation_2d"
    bl_label = "Delaunay triangulation 2½D"
    bl_space_type = "VIEW_3D"
    bl_region_type = "UI"
    bl_category = "Hallr tools"

    def draw(self, context):
        if context.mode != 'OBJECT':
            return

        layout = self.layout

        row = layout.row(align=True)
        # Bounding shape selection
        # row.label(text="Bounding Shape:")
        if context.scene.hallr_dt2_delaunay_settings.bounding_shape is not None:
            row.operator(HALLR_OT_D2TSelectBoundingShape.bl_idname, text="De-Select Bounding Shape", icon='X')
        else:
            row.operator(HALLR_OT_D2TSelectBoundingShape.bl_idname, text="Select Bounding Shape", icon='EYEDROPPER')
        if context.scene.hallr_dt2_delaunay_settings.bounding_shape is not None:
            row.label(text=context.scene.hallr_dt2_delaunay_settings.bounding_shape.name, icon='CHECKMARK')

        if context.scene.hallr_dt2_delaunay_settings.bounding_shape is not None:
            row = layout.row(align=True)
            row.prop(context.scene.hallr_dt2_delaunay_settings, "bounds_props")

        row = layout.row(align=True)
        # 3D mesh/point cloud for height offsets
        if context.scene.hallr_dt2_delaunay_settings.point_cloud is not None:
            row.operator(HALLR_OT_D2TSelectPointCloud.bl_idname, text="De-Select Point cloud", icon='X')
        else:
            row.operator(HALLR_OT_D2TSelectPointCloud.bl_idname, text="Select Point cloud", icon='EYEDROPPER')

        if context.scene.hallr_dt2_delaunay_settings.point_cloud:
            row.label(text=context.scene.hallr_dt2_delaunay_settings.point_cloud.name, icon='CHECKMARK')

        # Generate toolpath button
        if (context.scene.hallr_dt2_delaunay_settings.bounding_shape is not None and
                context.scene.hallr_dt2_delaunay_settings.point_cloud is not None):
            layout.operator(HALLR_OT_DT2GenerateMesh.bl_idname, text="Generate Mesh")


class HALLR_OT_D2TSelectBoundingShape(bpy.types.Operator):
    bl_idname = "hallr.dt2_select_bounding_shape"
    bl_label = "Select Bounding Shape"
    bl_description = "Select or deselect the bounding shape for triangulation"
    bl_context = "object"

    def execute(self, context):
        settings = context.scene.hallr_dt2_delaunay_settings
        if settings.bounding_shape is not None:
            settings.bounding_shape = None
            return {'FINISHED'}

        # Check the bounding shape
        bounding_shape = bpy.context.active_object
        if bounding_shape.type != 'MESH':
            self.report({'ERROR'}, "The bounding shape should be of type 'MESH'.")
            settings.bounding_shape = None
            return {'CANCELLED'}

        if settings.point_cloud is not None and bpy.context.active_object == settings.point_cloud:
            self.report({'ERROR'},
                        "This object is already selected as the point cloud. Please select a different object.")
            return {'CANCELLED'}

        settings.bounding_shape = bounding_shape
        return {'FINISHED'}


class HALLR_OT_D2TSelectPointCloud(bpy.types.Operator):
    bl_idname = "hallr.dt2_select_point_cloud"
    bl_label = "Select point cloud"
    bl_description = "Select or deselect the point cloud object for triangulation"
    bl_context = "object"

    def execute(self, context):
        settings = context.scene.hallr_dt2_delaunay_settings
        if settings.point_cloud is not None:
            settings.point_cloud = None
            return {'FINISHED'}

        if bpy.context.active_object.type != 'MESH':
            self.report({'ERROR'}, "The point cloud should be of type 'MESH'.")
            settings.point_cloud = None
            return {'CANCELLED'}

        if settings.bounding_shape is not None and bpy.context.active_object == settings.bounding_shape:
            self.report({'ERROR'},
                        "This object is already selected as the bounding shape. Please select a different object.")
            return {'CANCELLED'}

        settings.point_cloud = bpy.context.active_object
        return {'FINISHED'}


class HALLR_OT_DT2GenerateMesh(bpy.types.Operator):
    bl_idname = "hallr.d2t_generate_mesh"
    bl_label = "Generate 2½D Mesh"
    bl_context = "object"
    bl_description = "Create a 2½D Delaunay triangulation from the selected point cloud within the bounding shape"

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
                      hallr_ffi_utils.COMMAND_TAG: "2d_delaunay_triangulation"}
            try:
                # Call the Rust function
                _,info = hallr_ffi_utils.process_mesh_with_rust(config, primary_object=point_cloud,
                                                       secondary_object=bounding_shape,
                                                       primary_format=hallr_ffi_utils.MeshFormat.POINT_CLOUD,
                                                       secondary_format=hallr_ffi_utils.MeshFormat.POINT_CLOUD,
                                                       create_new=True)
                self.report({'INFO'}, info)
            except Exception as e:
                import traceback
                traceback.print_exc()
                self.report({'ERROR'}, f"Error: {e}")
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
    HALLR_OT_D2TSelectBoundingShape,
    HALLR_OT_D2TSelectPointCloud,
    HALLR_OT_DT2GenerateMesh,
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
