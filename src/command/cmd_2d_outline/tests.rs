// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, Model, OwnedModel},
    ffi::{MESH_FORMAT_TAG, MeshFormat},
};
use vector_traits::glam::Vec3;

#[test]
fn test_2d_outline_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string(),
    );
    let _ = config.insert("command".to_string(), "2d_outline".to_string());

    let owned_model = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.3121257, -0.5275663, 0.0).into(),
            (0.5275663, -1.3121257, 0.0).into(),
            (-0.5275663, 1.3121257, 0.0).into(),
            (1.3121257, 0.5275663, 0.0).into(),
        ],
        indices: vec![1, 2, 0, 1, 3, 2],
    };

    let model = Model {
        world_orientation: &owned_model.world_orientation,
        vertices: &owned_model.vertices,
        indices: &owned_model.indices,
    };
    let result = super::process_command::<Vec3>(config, vec![model])?;
    assert_eq!(8, result.1.len());
    assert_eq!(4, result.0.len());
    Ok(())
}

#[test]
fn test_2d_outline_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string(),
    );
    let _ = config.insert("command".to_string(), "2d_outline".to_string());

    let owned_model = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.0113943, -0.9753443, 0.0).into(),
            (1.0, -1.0, 0.0).into(),
            (-1.0, 1.0, 0.0).into(),
            (1.0241334, 1.0380125, 0.0).into(),
            (-0.13404018, 1.979902, 0.0).into(),
            (-1.8112676, -0.21234381, 0.0).into(),
            (1.5378065, -0.20696306, 0.0).into(),
            (1.5378065, -0.20696306, 0.0).into(),
        ],
        indices: vec![1, 2, 0, 3, 4, 2, 0, 2, 5, 1, 3, 2, 7, 1, 6, 7, 3, 1],
    };

    let model = Model {
        world_orientation: &owned_model.world_orientation,
        vertices: &owned_model.vertices,
        indices: &owned_model.indices,
    };
    let result = super::process_command::<Vec3>(config, vec![model])?;
    assert_eq!(16, result.1.len());
    assert_eq!(8, result.0.len());
    Ok(())
}
