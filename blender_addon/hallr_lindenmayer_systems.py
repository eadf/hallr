import bpy

# Define L-System Presets
L_SYSTEM_PRESETS = {
    "DRAGON_CURVE": '''# Dragon Curve L-System
axiom = "FX"
rules = {
    "X": "X+YF+",
    "Y": "-FX-Y"
}
angle = 90
iterations = 10
''',
    "KOCH_CURVE": '''# Koch Curve L-System
axiom = "F"
rules = {
    "F": "F+F-F-F+F"
}
angle = 90
iterations = 4
''',
    "SIERPINSKI_TRIANGLE": '''# Sierpinski Triangle L-System
axiom = "F-G-G"
rules = {
    "F": "F-G+F+G-F",
    "G": "GG"
}
angle = 120
iterations = 5
''',
}

# Function to generate dropdown items dynamically
def get_lsystem_presets(self, context):
    return [(key, key.replace("_", " "), f"Generate a {key.replace('_', ' ')}") for key in L_SYSTEM_PRESETS.keys()]

# Define property correctly with default index instead of a string
bpy.types.Scene.lsystem_preset = bpy.props.EnumProperty(
    name="Preset",
    description="Choose an L-System preset",
    items=get_lsystem_presets,
    default=0  # Must be an index, not a string!
)

class LoadLSystemPresetOperator(bpy.types.Operator):
    """Loads an L-System preset into a new Text Editor file"""
    bl_idname = "script.hallr_load_lsystem_preset"
    bl_label = "Load L-System Preset"

    def execute(self, context):
        preset_name = context.scene.lsystem_preset  # Get selected preset
        preset_script = L_SYSTEM_PRESETS.get(preset_name, "")

        # Create a new text data block in the Text Editor
        text_data = bpy.data.texts.new(name=f"LSystem_{preset_name}")
        text_data.write(preset_script)

        # Switch to the Scripting workspace and show the text
        for area in bpy.context.screen.areas:
            if area.type == 'TEXT_EDITOR':
                area.spaces.active.text = text_data
                break

        self.report({'INFO'}, f"Loaded preset: {preset_name}")
        return {'FINISHED'}


class RunLSystemScriptOperator(bpy.types.Operator):
    """Runs the currently loaded L-System script"""
    bl_idname = "script.hallr_run_lsystem"
    bl_label = "Run L-System Script"

    def execute(self, context):
        text = bpy.context.space_data.text  # Get the active text file
        if text:
            script_content = text.as_string()  # Get script as string
            try:
                print("****")
                print(script_content)
                print("****")
                #exec(script_content, {})  # Execute the script
                self.report({'INFO'}, "L-System script executed")
            except Exception as e:
                self.report({'ERROR'}, f"Error: {e}")
        else:
            self.report({'WARNING'}, "No script selected")

        return {'FINISHED'}

class LSystemPanel(bpy.types.Panel):
    """Creates a panel in the Sidebar (N Panel) for L-System presets"""
    bl_label = "L-System Presets"
    bl_idname = "VIEW3D_PT_hallr_lsystem_presets"
    bl_space_type = 'VIEW_3D'  # 3D Viewport
    bl_region_type = 'UI'  # Sidebar panel
    bl_category = "Hallr tools"

    def draw(self, context):
        layout = self.layout
        layout.prop(context.scene, "lsystem_preset")  # Dropdown menu
        layout.operator(LoadLSystemPresetOperator.bl_idname, text="Load Preset")

def draw_text_editor_button(self, context):
    layout = self.layout
    layout.operator(RunLSystemScriptOperator.bl_idname, text="Run L-System", icon="PLAY")

# define classes for registration
classes = (
    LoadLSystemPresetOperator,
    RunLSystemScriptOperator,
    LSystemPanel
)

def register():
    try:
        for cls in classes:
            bpy.utils.register_class(cls)
    except Exception as e:
        print(f"Failed to register operator: {e}")
        raise e

    bpy.types.TEXT_MT_editor_menus.append(draw_text_editor_button)  # Add button to Text Editor

def unregister():
    for cls in reversed(classes):
        try:
            bpy.utils.unregister_class(cls)
        except (RuntimeError, NameError):
            pass

    bpy.types.TEXT_MT_editor_menus.remove(draw_text_editor_button)  # Remove button on unregister
    del bpy.types.Scene.lsystem_preset

if __name__ == "__main__":
    register()
