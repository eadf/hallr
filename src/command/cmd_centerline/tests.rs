// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, Model, OwnedModel},
    ffi::MeshFormat,
};
use vector_traits::glam::Vec3;

#[test]
fn test_centerline_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("KEEP_INPUT".to_string(), "true".to_string());
    let _ = config.insert("NEGATIVE_RADIUS".to_string(), "true".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Edges.to_string(),
    );
    let _ = config.insert("WELD".to_string(), "true".to_string());
    let _ = config.insert("▶".to_string(), "centerline".to_string());
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
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Edges.to_string(),
    );
    let _ = config.insert("KEEP_INPUT".to_string(), "false".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.004999999888241291".to_string());
    let _ = config.insert("WELD".to_string(), "true".to_string());
    let _ = config.insert("▶".to_string(), "centerline".to_string());
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
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Edges.to_string(),
    );
    let _ = config.insert("▶".to_string(), "centerline".to_string());
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

#[test]
fn test_centerline_4() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("NEGATIVE_RADIUS".to_string(), "true".to_string());
    let _ = config.insert("≈".to_string(), "9.999999747378752e-05".to_string());
    let _ = config.insert("ANGLE".to_string(), "89.00000133828577".to_string());
    let _ = config.insert("SIMPLIFY".to_string(), "true".to_string());
    let _ = config.insert("▶".to_string(), "centerline".to_string());
    let _ = config.insert("REMOVE_INTERNALS".to_string(), "true".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.05000000074505806".to_string());
    let _ = config.insert("📦".to_string(), "⸗".to_string());
    let _ = config.insert("KEEP_INPUT".to_string(), "true".to_string());

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

    let result = super::process_command::<Vec3>(config, models);

    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_centerline_5() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("≈".to_string(), "9.999999747378752e-05".to_string());
    let _ = config.insert("📦".to_string(), "⸗".to_string());
    let _ = config.insert("ANGLE".to_string(), "89.00000133828577".to_string());
    let _ = config.insert("REMOVE_INTERNALS".to_string(), "true".to_string());
    let _ = config.insert("NEGATIVE_RADIUS".to_string(), "true".to_string());
    let _ = config.insert("SIMPLIFY".to_string(), "true".to_string());
    let _ = config.insert("KEEP_INPUT".to_string(), "true".to_string());
    let _ = config.insert("▶".to_string(), "centerline".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.10000000149011612".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-5.009332, -5.1599483, 0.0).into(),
            (2.0873826, -0.2498683, 0.0).into(),
            (-7.931281, 7.931281, 0.0).into(),
            (7.931281, 7.931281, 0.0).into(),
            (-0.09036956, 2.2078757, 0.0).into(),
            (-4.901717, 1.7058221, 0.0).into(),
            (-5.4246902, 4.7390633, 0.0).into(),
            (-1.8058567, 0.5055364, 0.0).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 4, 2, 3, 4, 6, 7, 7, 5, 6, 5],
    };

    let models = vec![owned_model_0.as_model()];

    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(34, result.0.len()); // vertices
    assert_eq!(70, result.1.len()); // indices
    Ok(())
}
