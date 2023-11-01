import bpy
from . import hallr_ffi_utils

# Define the choices for the search pattern property
bounding_props_items = [
    ("AABB", "Aabb", "Axis aligned bounding box"),
    ("CONVEX_HULL", "ConvexHull", "Convex hull bounds")
    # ("LINE", "LineHull", "Line hull bounds")
]


class OBJECT_OT_hallr_2d_delaunay_triangulation(bpy.types.Operator):
    """2½D Delaunay Triangulation, will use the XY plane to stitch together point clouds"""

    bl_idname = "object.hallr_2d_delaunay_triangulation"
    bl_label = "Hallr 2½D Delaunay Triangulation"
    bl_options = {'REGISTER', 'UNDO'}

    bounds_props: bpy.props.EnumProperty(
        name="Bounding box",
        description="Choose a bounding box",
        items=bounding_props_items,
        default="AABB"
    )

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def execute(self, context):
        try:
            # 1. Get the selected objects and verify counts:
            selected_objects = bpy.context.selected_objects
            if len(selected_objects) != 2:
                self.report({'ERROR'}, "Please select exactly two objects: the mesh and the bounding shape.")
                return {'CANCELLED'}

            active_obj = bpy.context.active_object
            if not active_obj:
                self.report({'ERROR'}, "Nothing selected")
                return {'CANCELLED'}
            if active_obj.type != 'MESH':
                self.report({'ERROR'}, "The selected object is not a mesh")
                return {'CANCELLED'}

            # Identify the bounding shape:
            bounding_shape = next((obj for obj in selected_objects if obj != active_obj), None)
            if not bounding_shape:
                self.report({'ERROR'}, "Failed to find the bounding shape.")
                return {'CANCELLED'}

            # 2. Verify the bounding shape type:
            if bounding_shape.type != 'MESH':
                self.report({'ERROR'}, "The bounding shape should be of type 'MESH'.")
                return {'CANCELLED'}

            bpy.context.view_layer.update()

            config = {"bounds": str(self.bounds_props),
                      "mesh.format": "point_cloud",
                      "command": "2d_delaunay_triangulation"}

            if config["bounds"] == "LINE":
                # Ensure the bounding shape doesn't have any faces:
                if len(bounding_shape.data.polygons) > 0:
                    self.report({'ERROR'}, "The bounding shape should not have faces. It should be a line object.")
                    return {'CANCELLED'}

                # Check the bounding shape
                if not hallr_ffi_utils.is_loop(bounding_shape.data):
                    self.report({'ERROR'}, "The bounding shape should be a continuous loop.")
                    return {'CANCELLED'}

            # Call the Rust function
            vertices, indices, config = hallr_ffi_utils.call_rust(config, active_obj, bounding_shape)

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

        except Exception as e:
            self.report({'ERROR'}, str(e))
            return {'CANCELLED'}

        return {'FINISHED'}

    def invoke(self, context, event):
        wm = context.window_manager
        return wm.invoke_props_dialog(self)

    def draw(self, context):
        layout = self.layout
        layout.label(text="Use the second object to define an axis aligned")
        layout.label(text="bounding box or a bounding convex hull.")
        layout.prop(self, "bounds_props")


def VIEW3D_MT_2d_delaunay_triangulation_menu_item(self, context):
    self.layout.operator(OBJECT_OT_hallr_2d_delaunay_triangulation.bl_idname)


def register():
    bpy.utils.register_class(OBJECT_OT_hallr_2d_delaunay_triangulation)
    bpy.types.VIEW3D_MT_object_convert.append(VIEW3D_MT_2d_delaunay_triangulation_menu_item)


def unregister():
    try:
        bpy.utils.unregister_class(OBJECT_OT_hallr_2d_delaunay_triangulation)
    except (RuntimeError, NameError):
        pass

    bpy.types.VIEW3D_MT_object_convert.remove(VIEW3D_MT_2d_delaunay_triangulation_menu_item)
    for f in bpy.types.VIEW3D_MT_mesh_add._dyn_ui_initialize():
        if f.__name__ == VIEW3D_MT_2d_delaunay_triangulation_menu_item.__name__:
            bpy.types.VIEW3D_MT_object_convert.remove(f)


if __name__ == "__main__":
    unregister()
    register()
