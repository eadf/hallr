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
    "version": (0, 1, 12),
}

DEV_MODE = False  # Set this to False for distribution

if DEV_MODE:
    addon_path = "HALLR__BLENDER_ADDON_PATH"  # Modify this path to point to your addon directory
    if addon_path not in sys.path:
        sys.path.append(addon_path)
# the string "from ." will be find-and-replaced with "" if run in DEV_MODE
from . import hallr_ffi_utils
from . import hallr_mesh_operators
from . import hallr_meander_toolpath
from . import hallr_2d_delaunay_triangulation
from . import hallr_baby_shark_operators
from . import hallr_lindenmayer_systems

# define modules for registration
modules = (
    hallr_baby_shark_operators,
    hallr_mesh_operators,
    hallr_meander_toolpath,
    hallr_2d_delaunay_triangulation,
    hallr_lindenmayer_systems,
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
