// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
    ffi::MeshFormat,
};

#[test]
fn test_voronoi_mesh_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("▶".to_string(), "voronoi_mesh".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.2864788911621093".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );
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
    assert_eq!(
        *result.3.get(MeshFormat::MESH_FORMAT_TAG).unwrap(),
        MeshFormat::Triangulated.to_string()
    );
    Ok(())
}

#[test]
fn test_voronoi_mesh_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("DISTANCE".to_string(), "0.2864788911621093".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );
    let _ = config.insert("▶".to_string(), "voronoi_mesh".to_string());

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
    assert_eq!(9, result.0.len()); // vertices
    assert_eq!(27, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_voronoi_mesh_3() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("DISTANCE".to_string(), "0.2864788911621093".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );
    let _ = config.insert("▶".to_string(), "voronoi_mesh".to_string());

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
    assert_eq!(19, result.0.len()); // vertices
    assert_eq!(96, result.1.len()); // indices
    Ok(())
}
#[test]
fn test_voronoi_mesh4() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("▶".to_string(), "voronoi_mesh".to_string());
    let _ = config.insert("NEGATIVE_RADIUS".to_string(), "true".to_string());
    let _ = config.insert("DISTANCE".to_string(), "1.0".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (1.203918, 1.203918, 0.0).into(),
            (-1.805877, 0.74801874, 0.0).into(),
            (0.0, -1.7025971, 0.0).into(),
            (-0.36410117, 0.33949375, 0.0).into(),
            (0.25582898, -0.17708552, 0.0).into(),
        ],
        indices: vec![0, 1, 2, 0, 1, 2],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;
    assert_eq!(18, result.0.len()); // vertices
    assert_eq!(87, result.1.len()); // indices
    Ok(())
}
