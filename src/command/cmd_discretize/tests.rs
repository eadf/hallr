// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
    ffi::MeshFormat,
};

#[test]
fn test_discretize_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Edges.to_string(),
    );
    let _ = config.insert("discretize_length".to_string(), "50.0".to_string());
    let _ = config.insert("▶".to_string(), "discretize".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (1.203918, 1.203918, 0.0).into(),
            (-1.805877, 0.74801874, 0.0).into(),
            (0.0, -1.7025971, 0.0).into(),
            (-0.36410117, 0.33949375, 0.0).into(),
            (0.25582898, -0.17708552, 0.0).into(),
            (-0.6682936, 5.8671384, 0.50151926).into(),
        ],
        indices: vec![0, 1, 2, 0, 1, 2, 2, 5],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;
    assert_eq!(8, result.0.len()); // vertices
    assert_eq!(12, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_discretize_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("discretize_length".to_string(), "30.0".to_string());
    let _ = config.insert("▶".to_string(), "discretize".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Edges.to_string(),
    );

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (1.203918, 1.203918, 0.0).into(),
            (-1.805877, 0.74801874, 0.0).into(),
            (0.0, -1.7025971, 0.0).into(),
            (-0.36410117, 0.33949375, 0.0).into(),
            (0.25582898, -0.17708552, 0.0).into(),
            (-0.33308586, 7.871808, 0.9538619).into(),
        ],
        indices: vec![0, 1, 2, 0, 1, 2, 2, 5],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;
    assert_eq!(12, result.0.len()); // vertices
    assert_eq!(20, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_discretize_3() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("▶".to_string(), "discretize".to_string());
    let _ = config.insert("discretize_length".to_string(), "25.0".to_string());
    let _ = config.insert("📦".to_string(), "⸗".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: [
            -0.8538463, 1.3261375, 0.0, 0.0, -1.3261375, -0.8538463, 0.0, 0.0, 0.0, 0.0, 1.5772426,
            0.0, 1.6485528, 1.8051357, 0.0, 1.0,
        ],
        vertices: vec![
            (2.9746904, 2.658982, 0.7762852).into(),
            (1.1762615, -0.37484813, 0.0).into(),
            (-0.5314311, 2.277427, 0.0).into(),
        ],
        indices: vec![1, 0, 2, 1, 0, 2],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;
    assert!(!_result.1.is_empty());

    assert_eq!(14, _result.0.len()); // vertices
    assert_eq!(28, _result.1.len()); // indices
    Ok(())
}
