// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError, command,
    command::{ConfigType, OwnedModel},
};

#[test]
fn test_sdf_mesh_2_5_saft_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("SDF_RADIUS_MULTIPLIER".to_string(), "1.0".to_string());
    let _ = config.insert("📦".to_string(), "⸗".to_string());
    let _ = config.insert("SDF_DIVISIONS".to_string(), "100".to_string());
    let _ = config.insert(
        "REMOVE_DOUBLES_THRESHOLD".to_string(),
        "9.999999747378752e-05".to_string(),
    );
    let _ = config.insert("▶".to_string(), "sdf_mesh_2½_saft".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (0.0, -1.0, 0.49217868).into(),
            (-1.0, 1.0, 0.0).into(),
            (1.0, 1.0, 0.0).into(),
        ],
        indices: vec![1, 0, 2, 1, 0, 2],
    };

    let models = vec![owned_model_0.as_model()];

    let result = super::process_command(config, models)?;
    command::test_3d_triangulated_mesh(&result);
    assert_eq!(33876, result.0.len()); // vertices
    assert_eq!(203244, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_sdf_mesh_2_5_saft_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("SDF_DIVISIONS".to_string(), "50".to_string());
    let _ = config.insert("SDF_RADIUS_MULTIPLIER".to_string(), "1.0".to_string());
    let _ = config.insert(
        "REMOVE_DOUBLES_THRESHOLD".to_string(),
        "9.999999747378752e-05".to_string(),
    );
    let _ = config.insert("📦".to_string(), "⸗".to_string());
    let _ = config.insert("▶".to_string(), "sdf_mesh_2½_saft".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (2.9746904, 2.658982, 0.7762852).into(),
            (1.1762615, -0.37484813, 0.0).into(),
            (-0.5314311, 2.277427, 0.0).into(),
        ],
        indices: vec![1, 0, 2, 1, 0, 2],
    };

    let models = vec![owned_model_0.as_model()];

    let result = super::process_command(config, models)?;
    command::test_3d_triangulated_mesh(&result);
    assert_eq!(6672, result.0.len()); // vertices
    assert_eq!(40020, result.1.len()); // indices
    Ok(())
}
