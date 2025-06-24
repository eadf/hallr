// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError, command,
    command::{ConfigType, OwnedModel},
};

#[test]
fn test_mesh_cleanup_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("📦".to_string(), "△".to_string());
    let _ = config.insert("▶".to_string(), "mesh_cleanup".to_string());
    let _ = config.insert("max_iterations".to_string(), "10".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.0, -1.0, -1.0).into(),
            (-1.0, -1.0, 1.0).into(),
            (-1.0, 1.0, -1.0).into(),
            (-1.0, 1.0, 1.0).into(),
            (1.0, -1.0, -1.0).into(),
            (1.0, -1.0, 1.0).into(),
            (1.0, 1.0, -1.0).into(),
            (1.0, 1.0, 1.0).into(),
        ],
        indices: vec![
            1, 2, 0, 3, 6, 2, 7, 4, 6, 5, 0, 4, 6, 0, 2, 3, 5, 7, 1, 3, 2, 3, 7, 6, 7, 5, 4, 5, 1,
            0, 6, 4, 0, 3, 1, 5,
        ],
    };

    let models = vec![owned_model_0.as_model()];

    let result = super::process_command(config, models)?;
    command::test_3d_triangulated_mesh(&result);
    assert_eq!(8, result.0.len()); // vertices
    assert_eq!(36, result.1.len()); // indices
    Ok(())
}
