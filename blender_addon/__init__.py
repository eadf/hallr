import sys

bl_info = {
    "name": "Hallr",
    "blender": (3, 4, 1),
    "category": "Object",
    "description": "A collection of addons written in Rust",
    "author": "EAD",
    "version": (0, 1, 2),
    "warning": "This executes compiled rust code on your computer",
}

DEV_MODE = False  # Set this to False for distribution

if DEV_MODE:
    addon_path = "HALLR__BLENDER_ADDON_PATH"  # Modify this path to point to your addon directory
    if addon_path not in sys.path:
        sys.path.append(addon_path)
# the string "from ." will be find-and-replaced with "" if run in DEV_MODE
from . import hallr_collision
from . import hallr_ffi_utils
from . import hallr_simplify_rdp
from . import hallr_2d_delaunay_triangulation
from . import hallr_2d_outline
from . import hallr_centerline
from . import hallr_mesh_operators

# define modules for registration
modules = (
    hallr_mesh_operators,  # always register hallr_mesh_operators first
    hallr_collision,
    hallr_simplify_rdp,
    hallr_2d_delaunay_triangulation,
    hallr_2d_outline,
    hallr_centerline,
)


def register():
    for a_module in modules:
        a_module.register()


def unregister():
    for a_module in modules:
        a_module.unregister()


if __name__ == "__main__":
    unregister()  # Unregister everything

    import importlib

    # special treatment for the plain python module hallr_ffi_utils
    importlib.reload(hallr_ffi_utils)
    for module in modules:
        importlib.reload(module)
    register()  # Register everything again
