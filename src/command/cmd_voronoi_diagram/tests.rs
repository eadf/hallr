// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
    ffi::MeshFormat,
};

#[test]
fn test_voronoi_diagram_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("DISTANCE".to_string(), "1.0".to_string());
    let _ = config.insert("▶".to_string(), "voronoi_diagram".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Edges.to_string(),
    );
    let _ = config.insert("KEEP_INPUT".to_string(), "false".to_string());

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
    assert_eq!(16, result.0.len()); // vertices
    assert_eq!(32, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_voronoi_diagram_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("📦".to_string(), "⸗".to_string());
    let _ = config.insert("KEEP_INPUT".to_string(), "true".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.10000000149011612".to_string());
    let _ = config.insert("▶".to_string(), "voronoi_diagram".to_string());
    let _ = config.insert("≈".to_string(), "0.0010000000474974513".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-5.009332, -5.1599483, 0.0).into(),
            (2.0873826, -0.2498683, 0.0).into(),
            (-7.931281, 7.931281, 0.0).into(),
            (7.931281, 7.931281, 0.0).into(),
            (-0.09036956, 2.2078757, 0.0).into(),
            (-1.8058567, 0.5055364, 0.0).into(),
            (-4.901717, 1.7058221, 0.0).into(),
            (-5.4246902, 4.7390633, 0.0).into(),
            (3.1869242, 3.4316316, 0.0).into(),
            (-3.8383403, -2.4953904, 0.0).into(),
            (-1.8058567, 0.5055364, 0.0).into(),
            (-4.901717, 1.7058221, 0.0).into(),
        ],
        indices: vec![
            2, 0, 0, 1, 1, 3, 4, 2, 3, 4, 5, 10, 6, 11, 10, 11, 7, 10, 7, 11,
        ],
    };

    let models = vec![owned_model_0.as_model()];

    let result = super::process_command(config, models)?;
    assert_eq!(81, result.0.len()); // vertices
    assert_eq!(174, result.1.len()); // indices
    Ok(())
}
