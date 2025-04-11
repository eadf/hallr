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
timeout(1)
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
timeout(1)
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
timeout(1)
''',

    "Lévy curve 3d": '''
# build a crooked Lévy C curve in 3d
token("F", Turtle::Forward(1.0))
token("→", Turtle::Rotate(45.0, 0.0, 0.5))
token("←", Turtle::Rotate(-45.0, 0.0, -0.5))
axiom("F")
rule("F", "→ F ← ← F →")
iterations(12)
timeout(1)
''',

    'Sierpinski triangle': '''
# https://en.wikipedia.org/wiki/L-system#Examples_of_L-systems
# build a sierpinski triangle
token("F", Turtle::Forward(1.0))
token("G", Turtle::Forward(1.0))
token("→", Turtle::Yaw(120.0))
token("←", Turtle::Yaw(-120.0))
axiom("F←G←G")
rule("F", "F←G→F→G←F")
rule("G", "GG")
iterations(5)
timeout(1)
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
timeout(1)
''',

    'Sierpinski gasket 3d': '''
# build a sierpinski gasket in 3d
token("R", Turtle::Forward(1.0))
token("L", Turtle::Forward(1.0))
token("→", Turtle::Yaw(60.0))
token("←", Turtle::Rotate(-61.0, 0.0,1.0))
axiom("R")
rule("R", "L ← R ← L")
rule("L", "R → L → R")
iterations(8)
timeout(1)
''',

    "Gosper curve": '''
# Algorithmic_botany, page 12 (http://algorithmicbotany.org/papers/#abop)
# build gosper curve
token("R", Turtle::Forward(1.0))
token("L", Turtle::Forward(1.0))
token("→", Turtle::Yaw(60.0))
token("←", Turtle::Yaw(-60.0))
axiom("L")
rule("L", "L→R→→R←L←←LL←R→")
rule("R", "←L→RR→→R→L←←L←R")
iterations(3)
timeout(1)
''',

    "Koch curve": '''
# Algorithmic_botany, page 9 (http://algorithmicbotany.org/papers/#abop)
# build a koch curve
token("F", Turtle::Forward(1.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
axiom("F")
rule("F", "F → F ← F ← F → F")
iterations(5)
timeout(1)
''',

    "Koch snowflake": '''
# https://en.wikipedia.org/wiki/Koch_snowflake#Representation_as_Lindenmayer_system
token("F", Turtle::Forward(1.0))
token("→", Turtle::Yaw(60.0))
token("←", Turtle::Yaw(-60.0))
axiom("F←←F←←F")
rule("F", "F←F→F→→F←F")
# only on even iterations
iterations(4)
timeout(1)
''',

    "Koch curve 3d": '''
# build a koch curve in 3d
token("F", Turtle::Forward(1.0))
token("↑", Turtle::Pitch(90.0))
token("←", Turtle::Rotate(4.0, -90.0, 0.0))
axiom("F")
rule("F", " F ↑ F ← F ← F ↑ F")
iterations(5)
timeout(1)
''',

    "Quadratic Koch curve island": '''
# Algorithmic_botany, page 9 (http://algorithmicbotany.org/papers/#abop)
# build a quadratic koch curve island
token("F", Turtle::Forward(1.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
axiom("F←F←F←F")
rule("F", "F←F→F→FF←F←F→F")
#rule("F", " F→FF←FF←F←F→F→FF←F←F→F→FF→FF←F")
# caution: this example increases in size really fast
iterations(3)
timeout(1)
''',

    "Quadratic Koch curve island on a sphere": '''
# Quadratic Koch curve island on a sphere
token("F", Turtle::GeodesicForward(0.1))
# angle specially selected for tiling in iterations(3)
token("→", Turtle::GeodesicYaw(90.1175))
token("←", Turtle::GeodesicYaw(-90.0))
axiom("F←F←F←F")
rule("F", "F←F→F→FF←F←F→F")
# caution: this example increases in size really fast
iterations(3)
timeout(1)
geodesic_radius(5.0)
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
timeout(1)
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
timeout(1)
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
timeout(1)
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
rule("X","F → [ [ X ] ← X ] ← F [ ← F X ] → X" )
rule("F","F F")
# initial rotation 5° off Y axis
rotate(5.0,0.0,0.0)
iterations(6)
timeout(1)
''',

    "Fractal plant on a sphere": '''
# build a geodesic fractal plant
token("X", Turtle::Nop)
token("F", Turtle::GeodesicForward(0.05))
token("→", Turtle::GeodesicYaw(25.0))
token("←", Turtle::GeodesicYaw(-25.0))
token("[", Turtle::Push)
token("]", Turtle::Pop)
axiom("X")
rule("X","F → [ [ X ] ← X ] ← F [ ← F X ] → X" )
rule("F","F F")
iterations(7)
geodesic_radius(5.0)
timeout(1)
''',

    "Fractal plant 3d": '''
# https://en.wikipedia.org/wiki/L-system#Examples_of_L-systems
# build a fractal plant
token("X", Turtle::Nop)
token("F", Turtle::Forward(1.0))
token("→", Turtle::Rotate(25.0,0.0,-15.0))
token("←", Turtle::Rotate(-25.0,0.0,5.0))
token("[", Turtle::Push)
token("]", Turtle::Pop)
axiom("X")
rule("X","F → [ [ X ] ← X ] ← F [ ← F X ] → X" )
rule("F", "F F")
# initial rotation 5° off Y axis
rotate(5.0,0.0,0.0)
iterations(6)
timeout(1)
''',

    "Hilbert curve": '''
# build hilbert curve
# https://en.wikipedia.org/wiki/Hilbert_curve
token("A", Turtle::Nop)
token("B", Turtle::Nop)
token("F", Turtle::Forward(1.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
axiom("A")
rule("B", "←AF→BFB→FA←" )
rule("A", "→BF←AFA←FB→" )
iterations(5)
timeout(1)
''',

    "Hilbert curve 3d": '''
# build hilbert curve 3d
token("A", Turtle::Nop)
token("B", Turtle::Nop)
token("C", Turtle::Nop)
token("D", Turtle::Nop)
token("F", Turtle::Forward(1.0))
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
timeout(1)
''',

    "Hilbert curve 3d v2": '''
# build hilbert curve 3d version 2
token("X", Turtle::Nop)
token("F", Turtle::Forward(1.0))
token("↑", Turtle::Pitch(90.0))
token("↓", Turtle::Pitch(-90.0))
token("→", Turtle::Yaw(90.0))
token("←", Turtle::Yaw(-90.0))
token("↻", Turtle::Roll(90.0))
token("↺", Turtle::Roll(-90.0))
axiom("X")
rule("X", "↑↺XF↑↺XFX←F↑↻↻XFX↓F→↻↻XFX←F↻X←↻")
iterations(3)
timeout(1)
''',

    "Demo curve": '''
# you can use these chars as tokens =<>^?∧\→←↑↓↻↺↕↶↷⥀⥁⇐⇒⇑⇓⇕×∘+-[]
# and also [a-z][A-Z][0-9]

token("X", Turtle::Nop))
token("Y", Turtle::Nop))
token("F", Turtle::Forward(2))
token("⇑", Turtle::PenUp)
token("⇓", Turtle::PenDown)
token("→", Turtle::Yaw(120.0))
token("←", Turtle::Yaw(-120.0))
token("↑", Turtle::Pitch(1))
token("↓", Turtle::Pitch(-1.0))
token("↻", Turtle::Roll(1.0))
token("↺", Turtle::Roll(-1.0))
#  Rotate(yaw, pitch, roll) in that order
token("∘", Turtle::Rotate(-1.0, 2.0, 1.0))
axiom("F X")
rule("X","X ← Y F ←")
rule("Y","↑ ⇑ ↺ F ⇓ X ∘ ↑ ↻ Y")
iterations(3)
timeout(1)
# will round all float positions to the nearest integer
round()
'''}


# Function to generate dropdown items dynamically
def get_lsystem_presets(self, context):
    return [(key, key.replace("_", " "), f"Generate a {key.replace('_', ' ')}") for key in L_SYSTEM_PRESETS.keys()]


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

        self.report({'INFO'}, f"Loaded preset: {preset_name} in the scripting text editor")
        return {'FINISHED'}


class LSystemPresetPopupOperator(bpy.types.Operator):
    """Load a predefined L-System script into the Text Editor."""

    bl_idname = "script.hallr_preset_popup"
    bl_label = "Choose L-System Preset"

    def invoke(self, context, event):
        return context.window_manager.invoke_props_dialog(self, width=300)

    def draw(self, context):
        layout = self.layout
        layout.prop(context.scene, "lsystem_preset")

    def execute(self, context):
        bpy.ops.script.hallr_load_lsystem_preset()
        return {'FINISHED'}


class RunLSystemScriptOperator(bpy.types.Operator):
    """Run L-System script (view result in 3D View)."""

    bl_idname = "script.hallr_run_lsystem"
    bl_label = "Run L-System Script"
    bl_icon = "PLAY"

    def execute(self, context):
        text = bpy.context.space_data.text  # Get the active text file
        if text:
            script_content = text.as_string()  # Get script as string
            try:
                config = {
                    hallr_ffi_utils.COMMAND_TAG: "lsystems",
                    "🐢": script_content,
                }

                # Call the Rust function
                hallr_ffi_utils.process_config(config)

                self.report({'INFO'}, "L-System script executed")
            except Exception as e:
                import traceback
                traceback.print_exc()
                self.report({'ERROR'}, f"Error: {e}")
                return {'CANCELLED'}
        else:
            self.report({'WARNING'}, "No script selected")

        return {'FINISHED'}


def draw_text_editor_button(self, context):
    layout = self.layout
    row = layout.row(align=True)

    # Run button
    row.operator(RunLSystemScriptOperator.bl_idname, text="Run L-System", icon=RunLSystemScriptOperator.bl_icon)

    # Preset dropdown as a small icon button (file folder icon)
    row.operator("script.hallr_preset_popup", text="", icon='FILE_FOLDER')


# define classes for registration
classes = (
    LoadLSystemPresetOperator,
    RunLSystemScriptOperator,
    LSystemPresetPopupOperator
)


def register():
    try:
        for cls in classes:
            bpy.utils.register_class(cls)
    except Exception as e:
        print(f"Failed to register operator: {e}")
        raise e
    try:
        bpy.types.Scene.lsystem_preset = bpy.props.EnumProperty(
            name="Preset",
            description="Choose an L-System preset",
            items=get_lsystem_presets,
            default=0
        )
    except Exception as e:
        print(f"Failed to register scene property lsystem_preset: {e}")
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
