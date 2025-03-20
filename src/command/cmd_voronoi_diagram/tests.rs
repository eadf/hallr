// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
};

#[test]
fn test_voronoi_diagram_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("DISTANCE".to_string(), "1.0".to_string());
    let _ = config.insert("command".to_string(), "voronoi_diagram".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
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
    assert_eq!(18, result.0.len()); // vertices
    assert_eq!(32, result.1.len()); // indices
    Ok(())
}
