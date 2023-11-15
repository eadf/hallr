// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    command::{ConfigType, OwnedModel},
    HallrError,
};
use vector_traits::glam::Vec3;

#[test]
fn test_2d_delaunay_triangulation_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("first_index_model_0".to_string(), "0".to_string());
    let _ = config.insert("mesh.format".to_string(), "point_cloud".to_string());
    let _ = config.insert("bounds".to_string(), "AABB".to_string());
    let _ = config.insert(
        "command".to_string(),
        "2d_delaunay_triangulation".to_string(),
    );
    let _ = config.insert("first_vertex_model_1".to_string(), "13".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (0.05647427, 0.25491828, 0.0).into(),
            (0.124220066, 0.28291255, 0.0).into(),
            (0.2001399, 0.3328338, 0.06129472).into(),
            (0.20539124, 0.36586288, 0.0).into(),
            (0.20401457, 0.43980372, 0.0).into(),
            (0.13374856, 0.43236518, 0.0).into(),
            (0.11830258, 0.40931696, 0.0).into(),
            (0.0994954, 0.3760991, 0.0).into(),
            (0.09085787, 0.35842437, 0.03938318).into(),
            (0.079081185, 0.33115727, 0.0).into(),
            (0.06939726, 0.30474508, 0.0).into(),
            (0.06413481, 0.28769204, 0.0).into(),
            (0.05647427, 0.25491828, 0.0).into(),
        ],
        indices: vec![],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.8112676, -0.21234381, 0.0).into(),
            (-1.0113943, -0.9753443, 0.0).into(),
            (1.0, -1.0, 0.0).into(),
            (1.5378065, -0.20696306, 0.0).into(),
            (1.0241334, 1.0380125, 0.0).into(),
            (-0.13404018, 1.979902, 0.0).into(),
            (-1.0, 1.0, 0.0).into(),
            (-1.8112676, -0.21234381, 0.0).into(),
        ],
        indices: vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(16, result.0.len()); // vertices
    assert_eq!(78, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_2d_delaunay_triangulation_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        "command".to_string(),
        "2d_delaunay_triangulation".to_string(),
    );
    let _ = config.insert("first_vertex_model_1".to_string(), "13".to_string());
    let _ = config.insert("first_index_model_1".to_string(), "0".to_string());
    let _ = config.insert("mesh.format".to_string(), "point_cloud".to_string());
    let _ = config.insert("bounds".to_string(), "CONVEX_HULL".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (0.05647427, 0.25491828, 0.0).into(),
            (0.124220066, 0.28291255, 0.0).into(),
            (0.2001399, 0.3328338, 0.06129472).into(),
            (0.20539124, 0.36586288, 0.0).into(),
            (0.20401457, 0.43980372, 0.0).into(),
            (0.13374856, 0.43236518, 0.0).into(),
            (0.11830258, 0.40931696, 0.0).into(),
            (0.0994954, 0.3760991, 0.0).into(),
            (0.09085787, 0.35842437, 0.03938318).into(),
            (0.079081185, 0.33115727, 0.0).into(),
            (0.06939726, 0.30474508, 0.0).into(),
            (0.06413481, 0.28769204, 0.0).into(),
            (0.05647427, 0.25491828, 0.0).into(),
        ],
        indices: vec![],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.8112676, -0.21234381, 0.0).into(),
            (-1.0113943, -0.9753443, 0.0).into(),
            (1.0, -1.0, 0.0).into(),
            (1.5378065, -0.20696306, 0.0).into(),
            (1.0241334, 1.0380125, 0.0).into(),
            (-0.13404018, 1.979902, 0.0).into(),
            (-1.0, 1.0, 0.0).into(),
            (-1.8112676, -0.21234381, 0.0).into(),
        ],
        indices: vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(19, result.0.len()); // vertices
    assert_eq!(87, result.1.len()); // indices
    Ok(())
}
