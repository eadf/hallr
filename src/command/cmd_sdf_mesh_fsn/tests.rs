// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
    ffi::MeshFormat,
};

#[test]
fn test_sdf_mesh_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Edges.to_string(),
    );
    let _ = config.insert("▶".to_string(), "sdf_mesh".to_string());
    let _ = config.insert("SDF_DIVISIONS".to_string(), "50".to_string());
    let _ = config.insert("SDF_RADIUS_MULTIPLIER".to_string(), "1.0".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (1.203918, 1.203918, 1.0).into(),
            (-1.805877, 0.74801874, 0.0).into(),
            (0.0, -1.7025971, 0.0).into(),
            (-0.36410117, 0.33949375, -1.0).into(),
            (0.25582898, -0.17708552, 0.0).into(),
        ],
        indices: vec![0, 1, 2, 0, 1, 2],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;
    assert_eq!(973, result.0.len()); // vertices
    assert_eq!(3888, result.1.len()); // indices
    Ok(())
}
