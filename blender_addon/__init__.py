import bpy
import sys

DEV_MODE = False  # Set this to False for distribution

if DEV_MODE:
    addon_path = "HALLR__BLENDER_ADDON_PATH"  # Modify this path to point to your addon directory
    if addon_path not in sys.path:
        sys.path.append(addon_path)

try:
    from . import hallr_collision  # This is for the packaged addon
    from . import hallr_ffi_utils
    from . import hallr_convex_hull_2d
    from . import hallr_simplify_rdp
    from . import hallr_2d_delaunay_triangulation
except ImportError:
    import hallr_collision  # This is for direct run in Blender's text editor
    import hallr_ffi_utils
    import hallr_convex_hull_2d
    import hallr_simplify_rdp
    import hallr_2d_delaunay_triangulation
    
bl_info = {
    "name": "Hallr",
    "blender": (3, 4, 1),
    "category": "Object",
}


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
