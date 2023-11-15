// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    command::{ConfigType, Model, OwnedModel},
    HallrError,
};
use vector_traits::glam::Vec3;

#[test]
fn test_convex_hull_2d_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("command".to_string(), "convex_hull_2d".to_string());

    let owned_model = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.0, 1.0, 0.0).into(),
            (-1.8112676, -0.21234381, 0.0).into(),
            (1.0241334, 1.0380125, 0.0).into(),
            (-0.13404018, 1.979902, 0.0).into(),
            (-1.0113943, -0.9753443, 0.0).into(),
            (1.0, -1.0, 0.0).into(),
            (1.5378065, -0.20696306, 0.0).into(),
            (1.5378065, -0.20696306, 0.0).into(),
        ],
        indices: vec![],
    };

    let model = owned_model.as_model();
    let result = super::process_command::<Vec3>(config, vec![model])?;
    assert_eq!(8, result.0.len());
    assert_eq!(9, result.1.len());
    Ok(())
}

#[test]
fn test_convex_hull_2d_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("command".to_string(), "convex_hull_2d".to_string());

    let owned_model = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (0.2001399, 0.3328338, 0.0).into(),
            (0.18789414, 0.3487433, 0.0).into(),
            (0.17686963, 0.36596286, 0.0).into(),
            (0.16706635, 0.3844924, 0.0).into(),
            (0.15414335, 0.36228794, 0.0).into(),
            (0.1409539, 0.33191225, 0.0).into(),
            (0.124220066, 0.28291255, 0.0).into(),
            (0.05647427, 0.25491828, 0.0).into(),
            (0.06413481, 0.28769204, 0.0).into(),
            (0.06939726, 0.30474508, 0.0).into(),
            (0.079081185, 0.33115727, 0.0).into(),
            (0.09085787, 0.35842437, 0.0).into(),
            (0.0994954, 0.3760991, 0.0).into(),
            (0.11830258, 0.40931696, 0.0).into(),
            (0.13374856, 0.43236518, 0.0).into(),
            (0.20539124, 0.36586288, 0.0).into(),
            (0.19336753, 0.38696265, 0.0).into(),
            (0.18305355, 0.41007194, 0.0).into(),
            (0.20401457, 0.43980372, 0.0).into(),
        ],
        indices: vec![],
    };

    let result = super::process_command::<Vec3>(config, vec![owned_model.as_model()])?;
    assert_eq!(13, result.0.len());
    assert_eq!(14, result.1.len());
    Ok(())
}

#[test]
fn test_convex_hull_2d_3() -> Result<(), HallrError> {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    let mut config = ConfigType::default();
    let _ = config.insert("command".to_string(), "convex_hull_2d".to_string());

    let mut rng: StdRng = SeedableRng::from_seed([42; 32]);
    let mut owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    for _i in 0..3023 {
        owned_model_0.vertices.push(
            (
                rng.gen_range(-100_f32..100.0),
                rng.gen_range(-100_f32..100.0),
                0.0,
            )
                .into(),
        );
    }

    let model_0 = owned_model_0.as_model();
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    println!("vertices: {:?}", result.0);
    println!("indices: {:?}", result.1);
    assert_eq!(25, result.0.len()); // vertices
    assert_eq!(26, result.1.len()); // indices

    // test that the convex hull of the convex hull remain the same
    let mut config = ConfigType::default();
    let _ = config.insert("command".to_string(), "convex_hull_2d".to_string());
    let model_0 = Model {
        world_orientation: &owned_model_0.world_orientation,
        indices: &vec![],
        vertices: &result.0,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(25, result.0.len()); // vertices
    assert_eq!(26, result.1.len()); // indices
    Ok(())
}
