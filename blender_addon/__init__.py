import bpy
import sys

bl_info = {
    "name": "Hallr",
    "blender": (3, 4, 1),
    "category": "Object",
    "description": "A collection of addons written in rust",
    "author": "EAD",
    "version": (0, 1, 0),
    "warning": "This executes rust code on your computer",
}

DEV_MODE = False  # Set this to False for distribution

if DEV_MODE:
    addon_path = "HALLR__BLENDER_ADDON_PATH"  # Modify this path to point to your addon directory
    if addon_path not in sys.path:
        sys.path.append(addon_path)
# the string "from ." will be find-and-replaced with "" if run in DEV_MODE
from . import hallr_collision
from . import hallr_ffi_utils
from . import hallr_convex_hull_2d
from . import hallr_simplify_rdp
from . import hallr_2d_delaunay_triangulation
from . import hallr_2d_outline
from . import hallr_centerline


# define modules for registration
modules = (
    hallr_collision,
    hallr_convex_hull_2d,
    hallr_simplify_rdp,
    hallr_2d_delaunay_triangulation,
    hallr_2d_outline,
    hallr_centerline
)


def register():
    for module in modules:
        module.register()


def unregister():
    for module in modules:
        module.unregister()


if __name__ == "__main__":
    unregister()  # Unregister everything

    import importlib
    # special treatment for the non-bpy module hallr_ffi_utils
    importlib.reload(hallr_ffi_utils)
    for module in modules:
        importlib.reload(module)
    register()  # Register everything again
