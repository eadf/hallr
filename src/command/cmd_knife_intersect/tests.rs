// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
    ffi::MeshFormat,
};
use vector_traits::glam::Vec3;

#[test]
fn knife_intersect_0() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );
    let _ = config.insert("▶".to_string(), "knife_intersect".to_string());

    let owned_model = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            (0.5, 0.0, 0.0).into(),
            (-0.5, 1.0, 0.0).into(),
        ],
        indices: vec![2, 3, 0, 1],
    };

    let result = super::process_command::<Vec3>(config, vec![owned_model.as_model()])?;
    assert_eq!(8, result.1.len());
    assert_eq!(5, result.0.len());

    Ok(())
}

#[test]
fn knife_intersect_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );
    let _ = config.insert("▶".to_string(), "knife_intersect".to_string());

    let owned_model = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            (0.5, 0.0, 0.0).into(),
            (-0.5, 1.0, 0.0).into(),
            (0.22312534, 0.7802051, 0.0).into(),
        ],
        indices: vec![2, 3, 0, 1, 3, 4],
    };

    let result = super::process_command::<Vec3>(config, vec![owned_model.as_model()])?;
    assert_eq!(7, result.1.chunks(2).count());
    assert_eq!(14, result.1.len());
    assert_eq!(8, result.0.len());
    Ok(())
}

#[test]
fn knife_intersect_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("▶".to_string(), "knife_intersect".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );

    let owned_model = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (2.2804604, -0.43074003, 0.0).into(),
            (2.2597418, -0.37131825, 0.0).into(),
            (2.2812376, -0.3050532, 0.0).into(),
            (2.3017414, -0.3271383, 0.0).into(),
            (2.3000648, -0.2969905, 0.0).into(),
            (2.2596474, -0.39532602, 0.0).into(),
            (2.2511325, -0.37128425, 0.0).into(),
        ],
        indices: vec![6, 4, 5, 6, 2, 3, 1, 2, 4, 5, 0, 1, 3, 0],
    };

    let model = owned_model.as_model();
    let result = super::super::process_command(
        model.vertices,
        model.indices,
        model.world_orientation,
        config,
    )?;
    assert_eq!(30, result.1.len());
    assert_eq!(14, result.0.len());
    Ok(())
}

#[test]
fn knife_intersect_3() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("▶".to_string(), "knife_intersect".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::LineChunks.to_string(),
    );

    let owned_model = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (2.2656271, -0.4295425, 0.0).into(),
            (2.260498, -0.41503745, 0.0).into(),
            (2.2581666, -0.3986339, 0.0).into(),
            (2.2584887, -0.38063163, 0.0).into(),
            (2.260995, -0.36200488, 0.0).into(),
            (2.2724524, -0.32310265, 0.0).into(),
            (2.2845647, -0.2965711, 0.0).into(),
            (2.295481, -0.2774364, 0.0).into(),
            (2.3017414, -0.3271383, 0.0).into(),
            (2.3028445, -0.4084011, 0.0).into(),
            (2.3048415, -0.44343835, 0.0).into(),
            (2.3007102, -0.46321002, 0.0).into(),
            (2.2803931, -0.44949877, 0.0).into(),
            (2.270601, -0.43815786, 0.0).into(),
            (2.2915509, -0.3062911, 0.0).into(),
            (2.2931828, -0.31041005, 0.0).into(),
            (2.2913845, -0.31730452, 0.0).into(),
            (2.2509167, -0.39224458, 0.0).into(),
            (2.2456083, -0.3875053, 0.0).into(),
            (2.2566564, -0.35506323, 0.0).into(),
            (2.281992, -0.30685732, 0.0).into(),
            (2.2867885, -0.30359325, 0.0).into(),
        ],
        indices: vec![
            0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
            14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 14, 13, 0,
        ],
    };

    let result = super::process_command::<Vec3>(config, vec![owned_model.as_model()])?;
    assert_eq!(52, result.1.len());
    assert_eq!(26, result.0.len());
    Ok(())
}
