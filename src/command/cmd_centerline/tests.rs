// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, Model, OwnedModel},
};
use vector_traits::glam::Vec3;

#[test]
fn test_centerline_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("KEEP_INPUT".to_string(), "true".to_string());
    let _ = config.insert("NEGATIVE_RADIUS".to_string(), "true".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("WELD".to_string(), "true".to_string());
    let _ = config.insert("command".to_string(), "centerline".to_string());
    let _ = config.insert("REMOVE_INTERNALS".to_string(), "true".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.004999999888241291".to_string());
    let _ = config.insert("ANGLE".to_string(), "89.00000133828577".to_string());
    let _ = config.insert("SIMPLIFY".to_string(), "true".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.8870333, -0.39229375, 0.010461569).into(),
            (-0.3180092, -2.0773406, 0.010461569).into(),
            (2.680789, 0.5384001, 0.010461569).into(),
            (-0.4052546, 2.4733071, 0.010461569).into(),
        ],
        indices: vec![0, 3, 0, 1, 2, 1, 3, 2],
    };

    let model_0 = Model {
        world_orientation: &owned_model_0.world_orientation,
        indices: &owned_model_0.indices,
        vertices: &owned_model_0.vertices,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(7, result.0.len()); // vertices
    assert_eq!(18, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_centerline_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("NEGATIVE_RADIUS".to_string(), "false".to_string());
    let _ = config.insert("SIMPLIFY".to_string(), "true".to_string());
    let _ = config.insert("REMOVE_INTERNALS".to_string(), "true".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("KEEP_INPUT".to_string(), "false".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.004999999888241291".to_string());
    let _ = config.insert("WELD".to_string(), "true".to_string());
    let _ = config.insert("command".to_string(), "centerline".to_string());
    let _ = config.insert("ANGLE".to_string(), "89.00000133828577".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.8870333, -0.39229375, 0.010461569).into(),
            (-0.3180092, -2.0773406, 0.010461569).into(),
            (2.680789, 0.5384001, 0.010461569).into(),
            (-0.4052546, 2.4733071, 0.010461569).into(),
        ],
        indices: vec![0, 3, 0, 1, 2, 1, 3, 2],
    };

    let model_0 = Model {
        world_orientation: &owned_model_0.world_orientation,
        indices: &owned_model_0.indices,
        vertices: &owned_model_0.vertices,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(7, result.0.len()); // vertices
    assert_eq!(10, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_centerline_3() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("REMOVE_INTERNALS".to_string(), "true".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("command".to_string(), "centerline".to_string());
    let _ = config.insert("NEGATIVE_RADIUS".to_string(), "true".to_string());
    let _ = config.insert("first_index_model_0".to_string(), "0".to_string());
    let _ = config.insert("ANGLE".to_string(), "89.00000133828577".to_string());
    let _ = config.insert("KEEP_INPUT".to_string(), "true".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.004999999888241291".to_string());
    let _ = config.insert("SIMPLIFY".to_string(), "true".to_string());
    let _ = config.insert("WELD".to_string(), "true".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.49995, -0.7411614, 0.0).into(),
            (-0.39808625, 0.6156829, 0.0).into(),
            (1.3165288, -0.969334, 0.0).into(),
            (-0.08538532, -0.12297079, 0.0).into(),
            (0.09803593, 1.5797875, 0.0).into(),
        ],
        indices: vec![0, 1, 2, 4, 1, 4, 3, 2, 3, 0],
    };

    let model_0 = Model {
        world_orientation: &owned_model_0.world_orientation,
        indices: &owned_model_0.indices,
        vertices: &owned_model_0.vertices,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(21, result.0.len()); // vertices
    assert_eq!(44, result.1.len()); // indices
    Ok(())
}
