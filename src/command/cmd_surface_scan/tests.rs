// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
    ffi::MeshFormat,
};
use vector_traits::{approx::ulps_eq, glam::Vec3};

#[test]
fn test_surface_scan_1() -> Result<(), HallrError> {
    // this test should report the same Z value for all samples
    let mut config = ConfigType::default();
    let _ = config.insert("first_index_model_1".to_string(), "6".to_string());
    let _ = config.insert("probe_angle".to_string(), "1.5707963705062866".to_string());
    let _ = config.insert("first_vertex_model_1".to_string(), "4".to_string());
    let _ = config.insert("probe".to_string(), "TAPERED_END".to_string());
    let _ = config.insert("bounds".to_string(), "AABB".to_string());
    let _ = config.insert("probe_radius".to_string(), "0.5".to_string());
    let _ = config.insert("pattern".to_string(), "MEANDER".to_string());
    let _ = config.insert("minimum_z".to_string(), "0.0".to_string());
    let _ = config.insert("step".to_string(), "0.20000000298023224".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string() + &MeshFormat::PointCloud.to_string(),
    );
    let _ = config.insert("▶".to_string(), "surface_scan".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.0, 0.00000004371139, 0.0).into(),
            (-0.018718276, 0.94025254, 0.6938799).into(),
            (1.0, -0.00000004371139, 0.0).into(),
            (0.0, 0.0, 0.0).into(),
        ],
        indices: vec![1, 2, 3, 1, 3, 0],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-0.61771935, 0.23340724, 0.009143627).into(),
            (0.5940437, 0.2347466, 0.009143627).into(),
            (-0.6233133, 0.5235412, 0.009143627).into(),
            (0.5884497, 0.5248806, 0.009143627).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 3, 2],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];

    let _result = super::process_command::<Vec3>(config, models)?;
    assert!(!_result.1.is_empty());
    for p in _result.0.iter() {
        // all z samples should be the same
        assert!(ulps_eq!(p.z, 0.17224793));
    }
    assert_eq!(7, _result.0.len()); // vertices
    assert_eq!(7, _result.1.len()); // indices
    Ok(())
}

#[test]
fn test_surface_scan_10() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("bounds".to_string(), "AABB".to_string());
    let _ = config.insert("probe_radius".to_string(), "0.5".to_string());
    let _ = config.insert("minimum_z".to_string(), "0.0".to_string());
    let _ = config.insert("first_index_model_1".to_string(), "15".to_string());
    let _ = config.insert("step".to_string(), "0.5".to_string());
    let _ = config.insert("▶".to_string(), "surface_scan".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string() + &MeshFormat::PointCloud.to_string(),
    );
    let _ = config.insert("pattern".to_string(), "MEANDER".to_string());
    let _ = config.insert("first_vertex_model_1".to_string(), "6".to_string());
    let _ = config.insert("probe".to_string(), "BALL_NOSE".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-0.29610628, -1.7045903, -0.9548358).into(),
            (-0.18138881, -0.23321122, 0.5500126).into(),
            (-1.5054786, 0.84019524, -0.70687366).into(),
            (1.5054786, -0.84019524, -1.0391741).into(),
            (0.6572089, 0.07475242, 0.09592825).into(),
            (0.29610628, 1.7045903, -0.79121196).into(),
        ],
        indices: vec![1, 2, 0, 3, 1, 0, 5, 1, 4, 3, 4, 1, 5, 2, 1],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.8112676, -0.21234381, 0.0).into(),
            (-1.0113943, -0.9753443, 0.0).into(),
            (1.0, -1.0, 0.0).into(),
            (1.5378065, -0.20696306, 0.0).into(),
            (1.0241334, 1.0380125, 0.0).into(),
            (-0.13404018, 1.979902, 0.0).into(),
            (-1.0, 1.0, 0.0).into(),
            (-1.8112676, -0.21234381, 0.0).into(),
        ],
        indices: vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(35, result.0.len()); // vertices
    assert_eq!(35, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_surface_scan_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("first_vertex_model_1".to_string(), "6".to_string());
    let _ = config.insert("pattern".to_string(), "MEANDER".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string() + &MeshFormat::PointCloud.to_string(),
    );
    let _ = config.insert("minimum_z".to_string(), "0.0".to_string());
    let _ = config.insert("▶".to_string(), "surface_scan".to_string());
    let _ = config.insert("probe_radius".to_string(), "0.5".to_string());
    let _ = config.insert("probe".to_string(), "BALL_NOSE".to_string());
    let _ = config.insert("step".to_string(), "0.5".to_string());
    let _ = config.insert("first_index_model_1".to_string(), "15".to_string());
    let _ = config.insert("bounds".to_string(), "CONVEX_HULL".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-0.29610628, -1.7045903, -0.9548358).into(),
            (-0.18138881, -0.23321122, 0.5500126).into(),
            (-1.5054786, 0.84019524, -0.70687366).into(),
            (1.5054786, -0.84019524, -1.0391741).into(),
            (0.6572089, 0.07475242, 0.09592825).into(),
            (0.29610628, 1.7045903, -0.79121196).into(),
        ],
        indices: vec![1, 2, 0, 3, 1, 0, 5, 1, 4, 3, 4, 1, 5, 2, 1],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.8112676, -0.21234381, 0.0).into(),
            (-1.0113943, -0.9753443, 0.0).into(),
            (1.0, -1.0, 0.0).into(),
            (1.5378065, -0.20696306, 0.0).into(),
            (1.0241334, 1.0380125, 0.0).into(),
            (-0.13404018, 1.979902, 0.0).into(),
            (-1.0, 1.0, 0.0).into(),
            (-1.8112676, -0.21234381, 0.0).into(),
        ],
        indices: vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 0],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(24, result.0.len()); // vertices
    assert_eq!(24, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_surface_scan_3() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("bounds".to_string(), "AABB".to_string());
    let _ = config.insert("first_vertex_model_1".to_string(), "5".to_string());
    let _ = config.insert("minimum_z".to_string(), "0.0".to_string());
    let _ = config.insert("probe_radius".to_string(), "0.5".to_string());
    let _ = config.insert("first_index_model_1".to_string(), "9".to_string());
    let _ = config.insert("step".to_string(), "0.5".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string() + &MeshFormat::PointCloud.to_string(),
    );
    let _ = config.insert("pattern".to_string(), "TRIANGULATION".to_string());
    let _ = config.insert("▶".to_string(), "surface_scan".to_string());
    let _ = config.insert("probe".to_string(), "BALL_NOSE".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.49995, -0.7401614, -0.66466707).into(),
            (-0.39808625, 0.6056829, 0.09412134).into(),
            (1.3165288, -0.969334, -0.54249233).into(),
            (-0.08538532, -0.1297079, 0.6106186).into(),
            (0.09803593, 1.5797875, -0.41113585).into(),
        ],
        indices: vec![4, 3, 2, 1, 0, 3, 1, 3, 4],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.3491066, -0.42415974, 0.0).into(),
            (0.42415974, -1.3491066, 0.0).into(),
            (-0.42415974, 1.3491066, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 3, 2],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(32, result.0.len()); // vertices
    assert_eq!(138, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_surface_scan_5() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        "xy_sample_dist_multiplier".to_string(),
        "0.4399999976158142".to_string(),
    );
    let _ = config.insert("probe_radius".to_string(), "0.5".to_string());
    let _ = config.insert("reduce_adaptive".to_string(), "true".to_string());
    let _ = config.insert("first_vertex_model_1".to_string(), "8".to_string());
    let _ = config.insert("first_index_model_1".to_string(), "24".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string(),
    );
    let _ = config.insert("probe".to_string(), "BALL_NOSE".to_string());
    let _ = config.insert("minimum_z".to_string(), "0.0".to_string());
    let _ = config.insert("bounds".to_string(), "CONVEX_HULL".to_string());
    let _ = config.insert("pattern".to_string(), "TRIANGULATION".to_string());
    let _ = config.insert("step".to_string(), "0.5".to_string());
    let _ = config.insert(
        "z_jump_threshold_multiplier".to_string(),
        "0.4399999976158142".to_string(),
    );
    let _ = config.insert("▶".to_string(), "surface_scan".to_string());

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
            0, 4, 6, 2, 3, 2, 6, 7, 7, 6, 4, 5, 5, 1, 3, 7, 1, 0, 2, 3, 5, 4, 0, 1,
        ],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-0.56271136, -2.59162, 0.0).into(),
            (2.59162, -0.56271136, 0.0).into(),
            (-2.59162, 0.56271136, 0.0).into(),
            (0.56271136, 2.59162, 0.0).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 3, 2],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let result = super::process_command::<Vec3>(config, models);
    assert!(result.is_err(), "Expected an error, but got Ok");

    Ok(())
}

#[test]
fn test_surface_scan_6() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("bounds".to_string(), "CONVEX_HULL".to_string());
    let _ = config.insert("step".to_string(), "1.0".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string() + &MeshFormat::PointCloud.to_string(),
    );
    let _ = config.insert("minimum_z".to_string(), "0.0".to_string());
    let _ = config.insert("▶".to_string(), "surface_scan".to_string());
    let _ = config.insert(
        "probe_radius".to_string(),
        "0.10000000149011612".to_string(),
    );
    let _ = config.insert("probe_angle".to_string(), "1.5707964".to_string()); // 90 degrees
    let _ = config.insert("first_vertex_model_1".to_string(), "7".to_string());
    let _ = config.insert("first_index_model_1".to_string(), "18".to_string());
    let _ = config.insert("probe".to_string(), "TAPERED_END".to_string());
    let _ = config.insert("pattern".to_string(), "MEANDER".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.9923778, -1.3102466, 0.49819893).into(),
            (-0.58778906, 0.27676046, 2.3478553).into(),
            (-1.8449209, 0.76535046, -1.3946409).into(),
            (-0.44033217, 2.3523574, 0.4550155).into(),
            (0.44033217, -2.3523574, -0.4550155).into(),
            (1.8449209, -0.76535046, 1.3946409).into(),
            (1.9923778, 1.3102466, -0.49819893).into(),
        ],
        indices: vec![1, 2, 0, 5, 1, 0, 1, 3, 2, 5, 0, 4, 3, 1, 5, 3, 5, 6],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (0.0, 3.8152, 0.0).into(),
            (-2.697754, 2.697754, 0.0).into(),
            (-3.8152, 0.0, 0.0).into(),
            (-2.697754, -2.697754, 0.0).into(),
            (0.0, -3.8152, 0.0).into(),
            (2.697754, -2.697754, 0.0).into(),
            (3.8152, 0.0, 0.0).into(),
            (2.697754, 2.697754, 0.0).into(),
        ],
        indices: vec![1, 0, 2, 1, 3, 2, 4, 3, 5, 4, 6, 5, 7, 6, 0, 7],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let _result = super::process_command::<Vec3>(config, models)?;
    //assert_eq!(0, result.0.len()); // vertices
    //assert_eq!(0, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_surface_scan_7() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("pattern".to_string(), "MEANDER".to_string());
    let _ = config.insert("probe_radius".to_string(), "0.5".to_string());
    let _ = config.insert("probe".to_string(), "TAPERED_END".to_string());
    let _ = config.insert("minimum_z".to_string(), "0.0".to_string());
    let _ = config.insert("first_vertex_model_1".to_string(), "3".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string() + &MeshFormat::PointCloud.to_string(),
    );
    let _ = config.insert("probe_angle".to_string(), "1.5707963705062866".to_string());
    let _ = config.insert("▶".to_string(), "surface_scan".to_string());
    let _ = config.insert("bounds".to_string(), "CONVEX_HULL".to_string());
    let _ = config.insert("step".to_string(), "1.0".to_string());
    let _ = config.insert("first_index_model_1".to_string(), "3".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.6849204, 2.8748128, 0.0).into(),
            (0.2528948, 3.6098597, 1.8858321).into(),
            (2.3150797, 2.8748128, 0.0).into(),
        ],
        indices: vec![2, 1, 0],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-2.446066, 4.005029, 0.0).into(),
            (-2.4750695, 0.0, 0.0).into(),
            (3.7157433, 0.0, 0.0).into(),
            (3.6867397, 4.005029, 0.0).into(),
        ],
        indices: vec![1, 0, 3, 2, 0, 3, 1, 2],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let _result = super::process_command::<Vec3>(config, models)?;
    //assert_eq!(0, result.0.len()); // vertices
    //assert_eq!(0, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_surface_scan_8() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("probe".to_string(), "TAPERED_END".to_string());
    let _ = config.insert("probe_angle".to_string(), "1.5707963705062866".to_string());
    let _ = config.insert("first_vertex_model_1".to_string(), "3".to_string());
    let _ = config.insert("minimum_z".to_string(), "0.0".to_string());
    let _ = config.insert("bounds".to_string(), "CONVEX_HULL".to_string());
    let _ = config.insert("step".to_string(), "1.0".to_string());
    let _ = config.insert("first_index_model_1".to_string(), "3".to_string());
    let _ = config.insert("pattern".to_string(), "MEANDER".to_string());
    let _ = config.insert("▶".to_string(), "surface_scan".to_string());
    let _ = config.insert("probe_radius".to_string(), "0.5".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string() + &MeshFormat::PointCloud.to_string(),
    );

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.0, 1.0, 0.0).into(),
            (0.0, -1.0, 1.0).into(),
            (1.0, 1.0, 0.0).into(),
        ],
        indices: vec![0, 1, 2],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-2.0, 1.0, 0.0).into(),
            (-2.0, 0.0, 0.0).into(),
            (3.0, 0.0, 0.0).into(),
            (3.0, 1.0, 0.0).into(),
        ],
        indices: vec![1, 0, 3, 2, 0, 3, 1, 2],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let _result = super::process_command::<Vec3>(config, models)?;
    //assert_eq!(0,result.0.len()); // vertices
    //assert_eq!(0,result.1.len()); // indices
    Ok(())
}

#[test]
fn test_surface_scan_9() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("first_vertex_model_1".to_string(), "9".to_string());
    let _ = config.insert("bounds".to_string(), "CONVEX_HULL".to_string());
    let _ = config.insert("probe_radius".to_string(), "0.5".to_string());
    //let _= config.insert("probe".to_string(),"TAPERED_END".to_string());
    let _ = config.insert("probe".to_string(), "SQUARE_END".to_string());
    let _ = config.insert("pattern".to_string(), "MEANDER".to_string());
    let _ = config.insert("step".to_string(), "0.03999999910593033".to_string());
    let _ = config.insert(
        MeshFormat::MESH_FORMAT_TAG.to_string(),
        MeshFormat::Triangulated.to_string() + &MeshFormat::PointCloud.to_string(),
    );
    let _ = config.insert(
        "xy_sample_dist_multiplier".to_string(),
        "0.10000000149011612".to_string(),
    );
    let _ = config.insert("first_index_model_1".to_string(), "9".to_string());
    let _ = config.insert("minimum_z".to_string(), "-1.0".to_string());
    let _ = config.insert("probe_angle".to_string(), "1.5707963705062866".to_string());
    let _ = config.insert("reduce_adaptive".to_string(), "false".to_string());
    let _ = config.insert(
        "z_jump_threshold_multiplier".to_string(),
        "0.10000000149011612".to_string(),
    );
    let _ = config.insert("▶".to_string(), "surface_scan".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-0.9426928, 0.665528, 0.7674321).into(),
            (-0.5, 0.30651796, 1.7009401).into(),
            (-0.057307184, 0.665528, 0.7674321).into(),
            (3.023047, -1.3744657, 0.0).into(),
            (2.023047, 0.12553431, 0.560689).into(),
            (1.023047, -1.3744656, 0.0).into(),
            (-0.7372743, -0.7912374, 0.75771683).into(),
            (-0.29458147, -0.32862473, 1.6912249).into(),
            (0.14811133, -0.7912374, 0.75771683).into(),
        ],
        indices: vec![0, 1, 2, 3, 4, 5, 7, 6, 8],
    };

    let owned_model_1 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-2.0, 2.0, 0.0).into(),
            (-2.0, -2.0, 0.0).into(),
            (3.0, -2.0, 0.0).into(),
            (3.0, 2.0, 0.0).into(),
        ],
        indices: vec![1, 0, 3, 2, 0, 3, 1, 2],
    };

    let models = vec![owned_model_0.as_model(), owned_model_1.as_model()];
    let _result = super::process_command::<Vec3>(config, models)?;
    //assert_eq!(0,result.0.len()); // vertices
    //assert_eq!(0,result.1.len()); // indices
    Ok(())
}
