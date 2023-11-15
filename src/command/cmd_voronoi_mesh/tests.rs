// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    command::{ConfigType, OwnedModel},
    HallrError,
};

#[test]
fn test_voronoi_mesh_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("command".to_string(), "voronoi_mesh".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.2864788911621093".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("first_index_model_0".to_string(), "0".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.3491066, -0.42415974, 0.0).into(),
            (0.42415974, -1.3491066, 0.0).into(),
            (-0.42415974, 1.3491066, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 3, 2],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;
    assert_eq!(5, result.0.len()); // vertices
    assert_eq!(12, result.1.len()); // indices
    assert_eq!("triangulated", result.3.get("mesh.format").unwrap());
    Ok(())
}

#[test]
fn test_voronoi_mesh_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("DISTANCE".to_string(), "0.2864788911621093".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("command".to_string(), "voronoi_mesh".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.3491066, -0.42415974, 0.0).into(),
            (0.42415974, -1.3491066, 0.0).into(),
            (-0.420259, 1.3558924, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
            (1.1850299, 1.4086196, 0.0).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 3, 2, 3, 4, 4, 5, 5, 2],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;
    assert_eq!(10, result.0.len()); // vertices
    assert_eq!(27, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_voronoi_mesh_3() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("DISTANCE".to_string(), "0.2864788911621093".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("command".to_string(), "voronoi_mesh".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.3491066, -0.42415974, 0.0).into(),
            (0.42415974, -1.3491066, 0.0).into(),
            (-0.420259, 1.3558924, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
            (-0.018198848, 0.30930626, 0.0).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 3, 2, 3, 4],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;
    assert_eq!(21, result.0.len()); // vertices
    assert_eq!(96, result.1.len()); // indices
    Ok(())
}
