"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import sys

bl_info = {
    "name": "Hallr",
    "blender": (3, 4, 1),
    "category": "Object",
    "description": "A collection of addons written in Rust",
    "author": "EAD https://github.com/eadf",
    "version": (0, 1, 19),
}

DEV_MODE = False  # Set this to False for distribution

try:
    if DEV_MODE:
        addon_path = "HALLR__BLENDER_ADDON_PATH"  # Modify this path to point to your addon directory
        if addon_path not in sys.path:
            sys.path.append(addon_path)

        import hallr_ffi_utils
        import hallr_mesh_operators
        import hallr_meander_toolpath
        import hallr_2d_delaunay_triangulation
        import hallr_lindenmayer_systems

    else:
        from . import hallr_ffi_utils
        from . import hallr_mesh_operators
        from . import hallr_meander_toolpath
        from . import hallr_2d_delaunay_triangulation
        from . import hallr_lindenmayer_systems

except Exception as e:
    print(f"=== MAIN INIT FAILED: {e} ===")
    import traceback

    traceback.print_exc()
    raise

# define modules for registration
modules = [
    hallr_ffi_utils,
    hallr_mesh_operators,
    hallr_meander_toolpath,
    hallr_2d_delaunay_triangulation,
    hallr_lindenmayer_systems,
]

def register():
    for a_module in modules:
        if hasattr(a_module, 'register'):
            a_module.register()


def unregister():
    for a_module in modules:
        if hasattr(a_module, 'unregister'):
            a_module.unregister()


def _reload_submodules(*submodules):
    """Call ._reload_submodules() on submodules"""
    import importlib
    for a_module in modules:
        if hasattr(a_module, '_reload_submodules'):
            a_module._reload_submodules()
        if DEV_MODE:
            print(f"reloading {a_module.__name__}")
        importlib.reload(a_module)


if __name__ == "__main__":
    unregister()  # Unregister everything
    _reload_submodules()
    register()  # Register everything again
