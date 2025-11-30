"""
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
This file is part of the hallr crate.
"""

import bpy
import sys

DEV_MODE = False  # Set this to False for distribution

if DEV_MODE:
    from hallr_mesh_operators.centerline import MESH_OT_hallr_centerline
    from hallr_mesh_operators.outline_2d import MESH_OT_hallr_2d_outline
    from hallr_mesh_operators.mesh_cleanup import MESH_OT_hallr_mesh_cleanup
    from hallr_mesh_operators.convex_hull import MESH_OT_hallr_convex_hull_2d
    from hallr_mesh_operators.isotropic_remesh import MESH_OT_hallr_isotropic_remesh
    from hallr_mesh_operators.knife_intersect import MESH_OT_hallr_knife_intersect
    from hallr_mesh_operators.simplify_rdp import MESH_OT_hallr_simplify_rdp
    from hallr_mesh_operators.voronoi_diagram import MESH_OT_hallr_voronoi_diagram
    from hallr_mesh_operators.voronoi_mesh import MESH_OT_hallr_voronoi_mesh
    from hallr_mesh_operators.sdf_mesh_25d import MESH_OT_hallr_sdf_mesh_25d
    from hallr_mesh_operators.sdf_mesh import MESH_OT_hallr_sdf_mesh
    from hallr_mesh_operators.discretize import MESH_OT_hallr_discretize
    from hallr_mesh_operators.pure_python import (MESH_OT_hallr_triangulate_and_flatten,
                                                  MESH_OT_hallr_random_vertices,
                                                  MESH_OT_hallr_select_collinear_edges,
                                                  MESH_OT_hallr_meta_volume,
                                                  MESH_OT_hallr_select_vertices_until_intersection,
                                                  MESH_OT_hallr_select_intersection_vertices,
                                                  MESH_OT_hallr_select_end_vertices)
else:

    from .centerline import MESH_OT_hallr_centerline
    from .outline_2d import MESH_OT_hallr_2d_outline
    from .mesh_cleanup import MESH_OT_hallr_mesh_cleanup
    from .convex_hull import MESH_OT_hallr_convex_hull_2d
    from .isotropic_remesh import MESH_OT_hallr_isotropic_remesh
    from .knife_intersect import MESH_OT_hallr_knife_intersect
    from .simplify_rdp import MESH_OT_hallr_simplify_rdp
    from .voronoi_diagram import MESH_OT_hallr_voronoi_diagram
    from .voronoi_mesh import MESH_OT_hallr_voronoi_mesh
    from .sdf_mesh_25d import MESH_OT_hallr_sdf_mesh_25d
    from .sdf_mesh import MESH_OT_hallr_sdf_mesh
    from .discretize import MESH_OT_hallr_discretize
    from .pure_python import (MESH_OT_hallr_triangulate_and_flatten,
                              MESH_OT_hallr_random_vertices,
                              MESH_OT_hallr_select_collinear_edges,
                              MESH_OT_hallr_meta_volume,
                              MESH_OT_hallr_select_vertices_until_intersection,
                              MESH_OT_hallr_select_intersection_vertices,
                              MESH_OT_hallr_select_end_vertices)


# menu containing all tools
class VIEW3D_MT_edit_mesh_hallr_meshtools(bpy.types.Menu):
    bl_label = "Hallr meshtools"

    def draw(self, context):
        layout = self.layout
        layout.operator(MESH_OT_hallr_2d_outline.bl_idname, icon=MESH_OT_hallr_2d_outline.bl_icon)
        layout.operator(MESH_OT_hallr_convex_hull_2d.bl_idname, icon=MESH_OT_hallr_convex_hull_2d.bl_icon)
        layout.operator(MESH_OT_hallr_voronoi_mesh.bl_idname, icon=MESH_OT_hallr_voronoi_mesh.bl_icon)
        layout.operator(MESH_OT_hallr_voronoi_diagram.bl_idname, icon=MESH_OT_hallr_voronoi_diagram.bl_icon)
        layout.operator(MESH_OT_hallr_sdf_mesh_25d.bl_idname, icon=MESH_OT_hallr_sdf_mesh_25d.bl_icon)
        layout.operator(MESH_OT_hallr_sdf_mesh.bl_idname, icon=MESH_OT_hallr_sdf_mesh.bl_icon)
        layout.operator(MESH_OT_hallr_meta_volume.bl_idname, icon=MESH_OT_hallr_meta_volume.bl_icon)
        layout.operator(MESH_OT_hallr_simplify_rdp.bl_idname, icon=MESH_OT_hallr_simplify_rdp.bl_icon)
        layout.operator(MESH_OT_hallr_centerline.bl_idname, icon=MESH_OT_hallr_centerline.bl_icon)
        layout.operator(MESH_OT_hallr_discretize.bl_idname, icon=MESH_OT_hallr_discretize.bl_icon)
        layout.operator(MESH_OT_hallr_isotropic_remesh.bl_idname, icon=MESH_OT_hallr_isotropic_remesh.bl_icon)
        layout.separator()
        layout.operator(MESH_OT_hallr_select_vertices_until_intersection.bl_idname,
                        icon=MESH_OT_hallr_select_vertices_until_intersection.bl_icon)
        layout.operator(MESH_OT_hallr_select_intersection_vertices.bl_idname,
                        icon=MESH_OT_hallr_select_intersection_vertices.bl_icon)
        layout.operator(MESH_OT_hallr_select_end_vertices.bl_idname, icon=MESH_OT_hallr_select_end_vertices.bl_icon)
        layout.operator(MESH_OT_hallr_select_collinear_edges.bl_idname,
                        icon=MESH_OT_hallr_select_collinear_edges.bl_icon)
        layout.operator(MESH_OT_hallr_knife_intersect.bl_idname, icon=MESH_OT_hallr_knife_intersect.bl_icon)
        layout.separator()
        layout.operator(MESH_OT_hallr_random_vertices.bl_idname, icon=MESH_OT_hallr_random_vertices.bl_icon)
        layout.operator(MESH_OT_hallr_triangulate_and_flatten.bl_idname,
                        icon=MESH_OT_hallr_triangulate_and_flatten.bl_icon),
        layout.operator(MESH_OT_hallr_mesh_cleanup.bl_idname,
                        icon=MESH_OT_hallr_mesh_cleanup.bl_icon)


# draw function for integration in menus
def menu_func(self, context):
    self.layout.menu("VIEW3D_MT_edit_mesh_hallr_meshtools")
    self.layout.separator()


# define classes for registration
classes = (
    VIEW3D_MT_edit_mesh_hallr_meshtools,
    MESH_OT_hallr_triangulate_and_flatten,
    MESH_OT_hallr_select_end_vertices,
    MESH_OT_hallr_select_intersection_vertices,
    MESH_OT_hallr_select_vertices_until_intersection,
    MESH_OT_hallr_select_collinear_edges,
    MESH_OT_hallr_convex_hull_2d,
    MESH_OT_hallr_knife_intersect,
    MESH_OT_hallr_voronoi_mesh,
    MESH_OT_hallr_voronoi_diagram,
    MESH_OT_hallr_sdf_mesh_25d,
    MESH_OT_hallr_sdf_mesh,
    MESH_OT_hallr_2d_outline,
    MESH_OT_hallr_simplify_rdp,
    MESH_OT_hallr_centerline,
    MESH_OT_hallr_discretize,
    MESH_OT_hallr_random_vertices,
    MESH_OT_hallr_meta_volume,
    MESH_OT_hallr_mesh_cleanup,
    MESH_OT_hallr_isotropic_remesh,
)

def _reload_submodules():
    """Call ._reload_submodules() on submodules and then reload it"""
    import importlib

    # Create a static list from the keys
    modules_to_reload = [module for module in sys.modules.keys() if 'hallr_mesh_operators.' in module]

    for module in modules_to_reload:
        if DEV_MODE:
            print(f"reloading {module}")
        importlib.reload(sys.modules[module])


# registering and menu integration
def register():
    try:
        for cls in classes:
            bpy.utils.register_class(cls)
    except Exception as e:
        print(f"Failed to register operator: {e}")
        raise e
    bpy.types.VIEW3D_MT_edit_mesh_context_menu.prepend(menu_func)


# unregistering and removing menus
def unregister():
    for cls in reversed(classes):
        try:
            bpy.utils.unregister_class(cls)
        except (RuntimeError, NameError):
            pass
    bpy.types.VIEW3D_MT_edit_mesh_context_menu.remove(menu_func)
