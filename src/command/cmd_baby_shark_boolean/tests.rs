// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError, command,
    command::{ConfigType, OwnedModel},
};

#[test]
fn test_baby_shark_boolean_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("operation".to_string(), "INTERSECT".to_string());
    let _ = config.insert("▶".to_string(), "baby_shark_boolean".to_string());
    let _ = config.insert("📦".to_string(), "△△".to_string());
    let _ = config.insert(
        "REMOVE_DOUBLES_THRESHOLD".to_string(),
        "9.999999747378752e-05".to_string(),
    );
    let _ = config.insert("first_vertex_model_1".to_string(), "8".to_string());
    let _ = config.insert("swap".to_string(), "False".to_string());
    let _ = config.insert("first_index_model_1".to_string(), "36".to_string());
    let _ = config.insert("voxel_size".to_string(), "0.5".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: [
            0.96372956,
            -0.20664234,
            -0.16889143,
            0.0,
            0.1811607,
            0.97122914,
            -0.15457936,
            0.0,
            0.19597492,
            0.1183762,
            0.97343767,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ],
        vertices: vec![
            (-1.3408651, -0.882963, -0.6499669).into(),
            (-0.94891536, -0.6462106, 1.2969085).into(),
            (-0.97854376, 1.0594953, -0.9591256).into(),
            (-0.5865939, 1.2962477, 0.98774976).into(),
            (0.5865939, -1.2962477, -0.98774976).into(),
            (0.97854376, -1.0594953, 0.9591256).into(),
            (0.94891536, 0.6462106, -1.2969085).into(),
            (1.3408651, 0.882963, 0.6499669).into(),
        ],
        indices: vec![
            1, 2, 0, 3, 6, 2, 7, 4, 6, 5, 0, 4, 6, 0, 2, 3, 5, 7, 1, 3, 2, 3, 7, 6, 7, 5, 4, 5, 1,
            0, 6, 4, 0, 3, 1, 5,
        ],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: [
            0.92953515,
            0.2425108,
            0.2777642,
            0.0,
            -0.29201552,
            0.944105,
            0.1529464,
            0.0,
            -0.22514744,
            -0.22328052,
            0.9483957,
            0.0,
            1.4313153,
            0.8895997,
            -0.17049451,
            1.0,
        ],
        vertices: vec![
            (1.0189431, -0.073735625, -1.5496008).into(),
            (0.5686482, -0.5202967, 0.34719062).into(),
            (0.4349121, 1.8144745, -1.243708).into(),
            (-0.015382811, 1.3679134, 0.65308344).into(),
            (2.8780134, 0.41128597, -0.99407244).into(),
            (2.4277186, -0.03527507, 0.902719).into(),
            (2.2939823, 2.299496, -0.6881796).into(),
            (1.8436875, 1.852935, 1.2086118).into(),
        ],
        indices: vec![
            1, 2, 0, 3, 6, 2, 7, 4, 6, 5, 0, 4, 6, 0, 2, 3, 5, 7, 1, 3, 2, 3, 7, 6, 7, 5, 4, 5, 1,
            0, 6, 4, 0, 3, 1, 5,
        ],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];

    let result = super::process_command(config, models)?;
    command::test_3d_triangulated_mesh(&result);
    assert_eq!(168, result.0.len()); // vertices
    assert_eq!(168, result.1.len()); // indices
    Ok(())
}
