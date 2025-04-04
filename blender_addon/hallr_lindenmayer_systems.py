import bpy
from . import hallr_ffi_utils

# Define L-System Presets
L_SYSTEM_PRESETS = {
    "Dragon curve": '''
# build_dragon_curve
# Algorithmic_botany, page 11 (http://algorithmicbotany.org/papers/#abop)
token("L", Turtle::Forward(1.0))
token("R", Turtle::Forward(1.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
axiom("L")
rule("L", "L → R →")
rule("R", "← L ← R")
iterations(8)
timeout(2)
''',

    "3d Dragon Curve": '''
# build dragon curve in 3d
token("X", Turtle::Nop)
token("Y", Turtle::Nop)
token("F", Turtle::Forward(1))
token("→", Turtle::Yaw(-90))
token("↑", Turtle::Pitch(90))
axiom("F X")
rule("X","X → Y F →")
rule("Y","↑ F X ↑ Y")
iterations(8)
timeout(2)
''',

    "Lévy curve": '''
# build a lévy curve
# https://en.wikipedia.org/wiki/Lévy_C_curve
token("F", Turtle::Forward(1.0))
token("→", Turtle::Yaw(45.0))
token("←", Turtle::Yaw(-45.0))
axiom("F")
rule("F", "→ F ← ← F →")
iterations(10)
timeout(2)
''',

    "Lévy curve 3d": '''
# build a crooked Lévy C curve in 3d
token("F", Turtle::Forward(1.0))
token("→", Turtle::Rotate(45.0, 0.0, 0.15))
token("←", Turtle::Rotate(-45.0, 0.0,-0.15))
axiom("F")
rule("F", "→ F ← ← F →")
iterations(12)
timeout(2)
''',

    'Sierpinski triangle': '''
# https://en.wikipedia.org/wiki/L-system#Examples_of_L-systems
# build a sierpinski triangle
token("F", Turtle::Forward(1.0))
token("G", Turtle::Forward(1.0))
token("→", Turtle::Yaw(120.0))
token("←", Turtle::Yaw(-120.0))
axiom("F←G←G")
rule("F", " F←G→F→G←F")
rule("G", " GG")
iterations(5)
timeout(2)
''',

    'Sierpinski gasket': '''
# Algorithmic_botany, page 11 (http://algorithmicbotany.org/papers/#abop)
# build sierpinski gasket
token("R", Turtle::Forward(1.0))
token("L", Turtle::Forward(1.0))
token("→", Turtle::Yaw(60.0))
token("←", Turtle::Yaw(-60.0))
axiom("R")
rule("R", " L ← R ← L")
rule("L", " R → L → R")
iterations(6)
timeout(2)
''',

    'Sierpinski gasket 3d': '''
# build a sierpinski gasket in 3d
token("R", Turtle::Forward(1.0))
token("L", Turtle::Forward(1.0))
token("→", Turtle::Yaw(60.0))
token("←", Turtle::Rotate(-61.0, 0.0,1.0))
axiom("R")
rule("R", " L ← R ← L")
rule("L", " R → L → R")
iterations(8)
timeout(2)
''',

    "Gosper curve": '''
# Algorithmic_botany, page 12 (http://algorithmicbotany.org/papers/#abop)
# build gosper curve
token("R", Turtle::Forward(1.0))
token("L", Turtle::Forward(1.0))
token("→", Turtle::Yaw(60.0))
token("←", Turtle::Yaw(-60.0))
axiom("L")
rule("L", " L→R→→R←L←←LL←R→")
rule("R", " ←L→RR→→R→L←←L←R")
iterations(3)
timeout(2)
''',

    "Koch curve": '''
# Algorithmic_botany, page 9 (http://algorithmicbotany.org/papers/#abop)
# build a koch curve
token("F", Turtle::Forward(1.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
axiom("F")
rule("F", " F → F ← F ← F → F")
iterations(5)
timeout(2)
''',

    "Koch curve 3d": '''
# build a koch curve in 3d
token("F", Turtle::Forward(1.0))
token("↑", Turtle::Pitch(90.0))
token("←", Turtle::Rotate(4.0, -90.0, 0.0))
axiom("F")
rule("F", " F ↑ F ← F ← F ↑ F")
iterations(5)
timeout(2)
''',

    "Quadratic Koch curve island": '''
# Algorithmic_botany, page 9 (http://algorithmicbotany.org/papers/#abop)
# build a quadratic koch curve island
token("F", Turtle::Forward(1.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
axiom("F←F←F←F")
rule("F", " F→FF←FF←F←F→F→FF←F←F→F→FF→FF←F")
# caution: this example increases in size really fast
iterations(3)
timeout(2)
''',

    "Quadratic Koch curve island 3d": '''
# build a quadratic koch curve island in 3d
token("F", Turtle::Forward(1.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
token("↻", Turtle::Roll(45.0))
token("↺", Turtle::Roll(-45.0))
axiom("F←F←F←F")
rule("F", " F→F↻F↺←F↻F↺←F←F→F→F↻F↺←F←F→F→F↻F↺→F↻F↺←F")
# caution: this example increases in size really fast
iterations(3)
timeout(2)
''',

    "Fractal binary tree": '''
# https://en.wikipedia.org/wiki/L-system#Examples_of_L-systems
# build fractal binary tree
token("0", Turtle::Forward(0.1))
token("1", Turtle::Forward(0.1))
token("→", Turtle::Yaw(45.0))
token("←", Turtle::Yaw(-45.0))
token("[", Turtle::Push)
token("]", Turtle::Pop)
axiom("0")
rule("1", " 11")
rule("0", " 1[→0]←0")
iterations(10)
timeout(2)
''',

    "Fractal binary tree 3d": '''
# build fractal binary tree in 3d
token("0", Turtle::Forward(0.1))
token("1", Turtle::Forward(0.1))
token("→", Turtle::Yaw(45.0))
token("←", Turtle::Yaw(-45.0))
token("↻", Turtle::Roll(15.0))
token("↺", Turtle::Roll(-15.0))
token("[", Turtle::Push)
token("]", Turtle::Pop)
axiom("0")
rule("1", "11")
rule("0", "1[→↺0]←↻0")
iterations(10)
timeout(2)
''',

    "Fractal plant": '''
# https://en.wikipedia.org/wiki/L-system#Examples_of_L-systems
# build a fractal plant
token("X", Turtle::Nop)
token("F", Turtle::Forward(1.0))
token("→", Turtle::Yaw(25.0))
token("←", Turtle::Yaw(-25.0))
token("[", Turtle::Push)
token("]", Turtle::Pop)
axiom("X")
rule("X"," F → [ [ X ] ← X ] ← F [ ← F X ] → X" )
rule("F", " F F")
iterations(6)
timeout(2)
''',

    "Fractal plant 3d": '''
# https://en.wikipedia.org/wiki/L-system#Examples_of_L-systems
# build a fractal plant
token("X", Turtle::Nop)
token("F", Turtle::Forward(1.0))
token("→", Turtle::Rotate(25.0,0.0,25.0))
token("←", Turtle::Rotate(-25.0,0.0, -25.0))
token("[", Turtle::Push)
token("]", Turtle::Pop)
axiom("X")
rule("X"," F → [ [ X ] ← X ] ← F [ ← F X ] → X" )
rule("F", " F F")
iterations(6)
timeout(2)
''',

    "Hilbert curve": '''
# build hilbert curve
# https://en.wikipedia.org/wiki/Hilbert_curve
token("A", Turtle::Nop)
token("B", Turtle::Nop)
token("F", Turtle::Forward(10.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
axiom("A")
rule("B", "←AF→BFB→FA←" )
rule("A", "→BF←AFA←FB→" )
iterations(5)
timeout(2)
''',

    "Hilbert curve 3d": '''
# build hilbert curve 3d
token("A", Turtle::Nop)
token("B", Turtle::Nop)
token("C", Turtle::Nop)
token("D", Turtle::Nop)
token("F", Turtle::Forward(10.0))
token("↑", Turtle::Pitch(90.0))
token("↓", Turtle::Pitch(-90.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
token("⇒", Turtle::Yaw(180.0))
token("↺", Turtle::Roll(-180.0))
axiom("A")
rule("A", " B←F→CFC→F←D↑F↓D←F→↑↑CFC→F→B↺")
rule("B", " A↑F↓CFB↓F↓D↓↓←F←D↓⇒F↓B⇒FC↓F↓A↺")
rule("C", " ⇒D↓⇒F↓B←F→C↓F↓A↑↑FA↑F↓C→F→B↓F↓D↺")
rule("D", " ⇒CFB←F→B⇒FA↑F↓A↑↑FB←F→B⇒FC↺")
iterations(3)
timeout(2)
''',

    "Hilbert curve 3d v2": '''
# build hilbert curve 3d version 2
token("X", Turtle::Nop)
token("F", Turtle::Forward(10.0))
token("↑", Turtle::Pitch(90.0))
token("↓", Turtle::Pitch(-90.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
token("↻", Turtle::Roll(90.0))
token("↺", Turtle::Roll(-90.0))
axiom("X")
rule("X", "↑↺XF↑↺XFX←F↑↻↻XFX↓F→↻↻XFX←F↻X←↻")
iterations(3)
timeout(2)
''',

    "custom curve": '''
token("X", Turtle::Nop))
token("Y", Turtle::Nop))
token("F", Turtle::Forward(1))
token("←", Turtle::Yaw(-90))
token("↑", Turtle::Pitch(90))
axiom("F X")
rule("X","X ← Y F ←")
rule("Y","↑ F X ↑ Y")
iterations(3)
timeout(2)
'''}


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
    bl_label = "Load L-System Preset in text editor"

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
    bl_icon = "PLAY"

    def execute(self, context):
        text = bpy.context.space_data.text  # Get the active text file
        if text:
            script_content = text.as_string()  # Get script as string
            try:
                config = {
                    "command": "lsystems",
                    "CUSTOM_TURTLE": script_content,
                }

                # Call the Rust function
                vertices, indices, config_out = hallr_ffi_utils.call_rust_direct(config, None)
                print("received:" + str(config_out))

                if config_out.get("ERROR"):
                    self.report({'ERROR'}, "" + config_out.get("ERROR"))
                    return {'CANCELLED'}

                hallr_ffi_utils.handle_chunks_line_new_object(config_out, vertices, indices)
                bpy.ops.object.mode_set(mode='OBJECT')

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
        layout.operator(LoadLSystemPresetOperator.bl_idname)


def draw_text_editor_button(self, context):
    layout = self.layout
    layout.operator(RunLSystemScriptOperator.bl_idname, text="Run L-System", icon=RunLSystemScriptOperator.bl_icon)


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
        except (RuntimeError, NameError, AttributeError):
            pass
    try:
        bpy.types.TEXT_MT_editor_menus.remove(draw_text_editor_button)  # Remove button on unregister
    except (RuntimeError, NameError, AttributeError):
        pass

    # Check if the attribute exists before trying to delete it
    if hasattr(bpy.types.Scene, 'lsystem_preset'):
        del bpy.types.Scene.lsystem_preset


if __name__ == "__main__":
    register()
