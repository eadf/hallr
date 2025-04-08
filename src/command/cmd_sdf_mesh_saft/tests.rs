// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
};

#[test]
fn test_sdf_mesh_saft_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        "REMOVE_DOUBLES_THRESHOLD".to_string(),
        "9.999999747378752e-05".to_string(),
    );
    let _ = config.insert("command".to_string(), "sdf_mesh_saft".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("SDF_RADIUS_MULTIPLIER".to_string(), "1.0".to_string());
    let _ = config.insert("SDF_DIVISIONS".to_string(), "50".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (1.0, 1.0, -1.0).into(),
            (-1.0, 1.0, 1.0).into(),
            (-1.0, 1.0, -1.0).into(),
        ],
        indices: vec![0, 2, 1, 2],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;
    assert_eq!(_result.1.len() % 3, 0);
    assert!(!_result.1.is_empty());
    let number_of_vertices = _result.0.len();
    assert!(number_of_vertices > 0);

    for t in _result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }
    assert_eq!(10444, _result.0.len()); // vertices
    assert_eq!(62652, _result.1.len()); // indices
    Ok(())
}
