// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
};

#[test]
fn test_baby_shark_mesh_offset_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("mesh.format".to_string(), "triangulated".to_string());
    let _ = config.insert("VOXEL_SIZE".to_string(), "1.0".to_string());
    let _ = config.insert("command".to_string(), "baby_shark_mesh_offset".to_string());
    let _ = config.insert("OFFSET_BY".to_string(), "1.5".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (1.0, 1.0, 1.0).into(),
            (1.0, 1.0, -1.0).into(),
            (1.0, -1.0, 1.0).into(),
            (1.0, -1.0, -1.0).into(),
            (-1.0, 1.0, 1.0).into(),
            (-1.0, 1.0, -1.0).into(),
            (-1.0, -1.0, 1.0).into(),
            (-1.0, -1.0, -1.0).into(),
        ],
        indices: vec![
            4, 2, 0, 2, 7, 3, 6, 5, 7, 1, 7, 5, 0, 3, 1, 4, 1, 5, 4, 6, 2, 2, 6, 7, 6, 4, 5, 1, 3,
            7, 0, 2, 3, 4, 0, 1,
        ],
    };

    let models = vec![owned_model_0.as_model()];

    let result = super::process_command(config, models)?;
    assert_eq!(result.1.len() % 3, 0);
    assert!(result.1.len() > 0);
    let number_of_vertices = result.0.len();
    assert!(number_of_vertices > 0);

    for t in result.1.chunks_exact(3) {
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
    //assert_eq!(0,result.0.len()); // vertices
    //assert_eq!(0,result.1.len()); // indices
    Ok(())
}
