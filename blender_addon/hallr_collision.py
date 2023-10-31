import bpy
from . import hallr_ffi_utils

# Define the choices for the tool/probe property
probes_props_items = [
    ("BALL_NOSE", "Ball Nose", "Ball Nose probe"),
    ("SQUARE_END", "Square End", "Square End probe")
]

# Define the choices for the search pattern property
patterns_props_items = [
    ("MEANDER", "Meander", "Meander scan pattern"),
    ("TRIANGULATION", "Triangulation", "2d Delaunay Triangulation")
]

# Define the choices for the search pattern property
bounding_props_items = [
    ("AABB", "Aabb", "Use secondary object as an Axis aligned bounding box"),
    ("CONVEX_HULL", "ConvexHull", "Use secondary object as Convex hull bounds")
    # ("LINE", "LineHull", "Line hull bounds")
]


class OBJECT_OT_hallr_collision(bpy.types.Operator):
    """A CNC toolpath/mesh probe operation, work in progress"""

    bl_idname = "object.hallr_collision"
    bl_label = "Hallr mesh toolpath generator"
    bl_options = {'REGISTER', 'UNDO'}

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
        max=10.0
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

    pattern_props: bpy.props.EnumProperty(
        name="Scan Pattern",
        description="Choose a scan pattern",
        items=patterns_props_items,
        default="MEANDER"
    )

    bounds_props: bpy.props.EnumProperty(
        name="Bounding box",
        description="Choose a bounding box",
        items=bounding_props_items,
        default="AABB"
    )

    def execute(self, context):
        try:
            # 1. Get the selected objects and verify counts:
            selected_objects = bpy.context.selected_objects
            if len(selected_objects) != 2:
                self.report({'ERROR'}, "Please select only two objects: the mesh and the bounding shape.")
                return {'CANCELLED'}

            active_obj = bpy.context.active_object
            if not active_obj:
                self.report({'ERROR'}, "Nothing selected")
                return {'CANCELLED'}
            if active_obj.type != 'MESH':
                self.report({'ERROR'}, "The selected object is not a mesh")
                return {'CANCELLED'}

            # Check if the mesh is triangulated
            for face in active_obj.data.polygons:
                if len(face.vertices) != 3:
                    raise ValueError("The mesh is not fully triangulated!")

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

            config = {"probe_radius": str(self.probe_radius_props), "probe": str(self.probe_props),
                      "bounds": str(self.bounds_props),
                      "pattern": str(self.pattern_props), "step": str(self.step_props),
                      "mesh.format": "triangulated", "minimum_z": str(self.minimum_z_props),
                      "command": "surface_scan"}
            if self.enable_adaptive_scan_props:
                config["z_jump_threshold_multiplier"] = str(self.z_jump_threshold_multiplier_props)
                config["xy_sample_dist_multiplier"] = str(self.xy_sample_dist_multiplier_props)
                config["reduce_adaptive"] = str(self.enable_reduce_props)

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

            # print(f"Received {config} as the result from Rust!")
            if config.get("ERROR"):
                self.report({'ERROR'}, "" + config.get("ERROR"))
                return {'CANCELLED'}
            # Check if the returned mesh format is triangulated
            if config.get("mesh.format") == "triangulated":
                hallr_ffi_utils.handle_triangle_mesh(vertices, indices)
            # Handle line format
            elif config.get("mesh.format") == "line":
                hallr_ffi_utils.handle_sliding_line_mesh(vertices, indices)
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
        layout.prop(self, "probe_props")
        layout.prop(self, "pattern_props")
        layout.prop(self, "bounds_props")
        layout.prop(self, "enable_adaptive_scan_props")
        if self.enable_adaptive_scan_props:
            layout.prop(self, "z_jump_threshold_multiplier_props")
            layout.prop(self, "xy_sample_dist_multiplier_props")
            layout.prop(self, "enable_reduce_props")
        layout.separator()
        layout.prop(self, "probe_radius_props")
        layout.prop(self, "step_props")
        layout.prop(self, "minimum_z_props")


def VIEW3D_MT_collision_menu_item(self, context):
    self.layout.operator(OBJECT_OT_hallr_collision.bl_idname)


def register():
    bpy.utils.register_class(OBJECT_OT_hallr_collision)
    bpy.types.VIEW3D_MT_object_convert.append(VIEW3D_MT_collision_menu_item)


def unregister():
    try:
        bpy.utils.unregister_class(OBJECT_OT_hallr_collision)
    except (RuntimeError, NameError):
        pass

    bpy.types.VIEW3D_MT_object_convert.remove(VIEW3D_MT_collision_menu_item)
    for f in bpy.types.VIEW3D_MT_object_convert._dyn_ui_initialize():
        if f.__name__ == VIEW3D_MT_collision_menu_item.__name__:
            bpy.types.VIEW3D_MT_object_convert.remove(f)


if __name__ == "__main__":
    unregister()
    register()
