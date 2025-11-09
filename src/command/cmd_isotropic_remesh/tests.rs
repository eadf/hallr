// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
    ffi::MeshFormat,
};

#[test]
fn test_isotropic_remesh_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("▶".to_string(), "isotropic_remesh".to_string());
    let _ = config.insert("COLLAPSE_EDGES".to_string(), "True".to_string());
    let _ = config.insert("FLIP_EDGES".to_string(), "VALENCE".to_string());
    let _ = config.insert(
        "TARGET_EDGE_LENGTH".to_string(),
        "0.10000000149011612".to_string(),
    );
    let _ = config.insert("SPLIT_EDGES".to_string(), "True".to_string());
    let _ = config.insert("ITERATIONS_COUNT".to_string(), "10".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string(),
    );

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (0.0, 0.11755705, 0.16180341).into(),
            (0.0, 0.19021131, 0.0618034).into(),
            (0.0, 0.19021131, -0.0618034).into(),
            (0.0, 0.11755705, -0.16180341).into(),
            (0.1118034, 0.036327124, 0.16180341).into(),
            (0.1809017, 0.058778524, 0.0618034).into(),
            (0.1809017, 0.058778524, -0.0618034).into(),
            (0.1118034, 0.036327124, -0.16180341).into(),
            (0.069098294, -0.095105655, 0.16180341).into(),
            (0.1118034, -0.15388419, 0.0618034).into(),
            (0.1118034, -0.15388419, -0.0618034).into(),
            (0.069098294, -0.095105655, -0.16180341).into(),
            (0.0, 0.0, 0.2).into(),
            (-0.06909831, -0.09510565, 0.16180341).into(),
            (-0.11180341, -0.15388417, 0.0618034).into(),
            (-0.11180341, -0.15388417, -0.0618034).into(),
            (-0.06909831, -0.09510565, -0.16180341).into(),
            (-0.1118034, 0.03632714, 0.16180341).into(),
            (-0.18090169, 0.058778543, 0.0618034).into(),
            (-0.18090169, 0.058778543, -0.0618034).into(),
            (-0.1118034, 0.03632714, -0.16180341).into(),
            (0.0, 0.0, -0.2).into(),
        ],
        indices: vec![
            2, 7, 3, 0, 5, 1, 21, 3, 7, 1, 6, 2, 0, 12, 4, 21, 7, 11, 5, 10, 6, 4, 12, 8, 6, 11, 7,
            4, 9, 5, 21, 11, 16, 9, 15, 10, 8, 12, 13, 11, 15, 16, 8, 14, 9, 21, 16, 20, 14, 19,
            15, 13, 12, 17, 15, 20, 16, 14, 17, 18, 21, 20, 3, 18, 2, 19, 17, 12, 0, 19, 3, 20, 18,
            0, 1, 2, 6, 7, 0, 4, 5, 1, 5, 6, 5, 9, 10, 6, 10, 11, 4, 8, 9, 9, 14, 15, 11, 10, 15,
            8, 13, 14, 14, 18, 19, 15, 19, 20, 14, 13, 17, 18, 1, 2, 19, 2, 3, 18, 17, 0,
        ],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;

    assert_eq!(result.1.len() % 3, 0);

    let number_of_vertices = result.0.len();
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
    //assert_eq!(33, result.0.len()); // vertices
    //assert_eq!(186, result.1.len()); // indices
    Ok(())
}
