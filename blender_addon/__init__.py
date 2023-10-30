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


def register():
    hallr_collision.register()
    hallr_convex_hull_2d.register()
    hallr_simplify_rdp.register()
    hallr_2d_delaunay_triangulation.register()


def unregister():
    hallr_collision.unregister()
    hallr_convex_hull_2d.unregister()
    hallr_simplify_rdp.unregister()
    hallr_2d_delaunay_triangulation.unregister()


if __name__ == "__main__":
    unregister()  # Unregister everything

    import importlib
    importlib.reload(hallr_collision)
    importlib.reload(hallr_ffi_utils)
    importlib.reload(hallr_convex_hull_2d)
    importlib.reload(hallr_simplify_rdp)
    importlib.reload(hallr_2d_delaunay_triangulation)
    register()  # Register everything again
