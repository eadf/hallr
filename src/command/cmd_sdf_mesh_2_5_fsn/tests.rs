// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError, command,
    command::{ConfigType, OwnedModel},
    ffi::MeshFormat,
};

#[test]
fn test_sdf_mesh_2_5_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("SDF_DIVISIONS".to_string(), "20".to_string());
    let _ = config.insert("SDF_RADIUS_MULTIPLIER".to_string(), "1.0".to_string());
    let _ = config.insert("▶".to_string(), "sdf_mesh_2_5".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.0, -1.0, 0.0).into(),
            (0.014304634, 0.021932945, 0.63773185).into(),
            (0.014304634, 0.021932945, 0.6377318).into(),
            (-0.48725998, 0.53284, 0.0).into(),
            (0.11475183, 0.05492184, 0.6363602).into(),
            (1.0, 1.0, 0.0).into(),
            (0.11475183, 0.05492184, 0.6363603).into(),
            (0.65058, -0.43409, 0.0).into(),
        ],
        indices: vec![0, 1, 2, 3, 1, 4, 4, 5, 6, 7],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;
    assert_eq!(1279, result.0.len()); // vertices
    assert_eq!(6384, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_sdf_mesh_2_5_fsn_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("SDF_RADIUS_MULTIPLIER".to_string(), "1.0".to_string());
    let _ = config.insert("▶".to_string(), "sdf_mesh_2½_fsn".to_string());
    let _ = config.insert("📦".to_string(), "⸗".to_string());
    let _ = config.insert(
        "REMOVE_DOUBLES_THRESHOLD".to_string(),
        "9.999999747378752e-05".to_string(),
    );
    let _ = config.insert("SDF_DIVISIONS".to_string(), "50".to_string());

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

    let result = super::process_command(config, models)?;
    command::test_3d_triangulated_mesh(&result);
    assert_eq!(4263, result.0.len()); // vertices
    assert_eq!(20964, result.1.len()); // indices
    Ok(())
}
