use crate::{
    command::{ConfigType, Model, OwnedModel},
    HallrError,
};
use vector_traits::glam::Vec3;

#[test]
fn test_simplify_rdp_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("command".to_string(), "simplify_rdp".to_string());
    let _ = config.insert("epsilon".to_string(), "0.20000000298023224".to_string());
    let _ = config.insert("simplify_3d".to_string(), "false".to_string());
    let _ = config.insert("first_vertex_model_0".to_string(), "0".to_string());
    let _ = config.insert("first_index_model_0".to_string(), "0".to_string());

    let owned_model_0 = OwnedModel {
        vertices: vec![
            (1.6574931, 1.296678, 0.0).into(),
            (1.6901442, 1.3938915, 0.0).into(),
            (1.6833773, 1.5016502, 0.0).into(),
            (1.6388826, 1.5919106, 0.0).into(),
            (1.5634191, 1.6562335, 0.0).into(),
            (1.4638305, 1.6880565, 0.0).into(),
            (1.3540487, 1.6814649, 0.0).into(),
            (1.2621217, 1.6380795, 0.0).into(),
            (1.196382, 1.5643816, 0.0).into(),
            (1.1637675, 1.4669337, 0.0).into(),
            (1.1705302, 1.3593122, 0.0).into(),
            (1.2149572, 1.2691299, 0.0).into(),
            (1.2901969, 1.2046038, 0.0).into(),
            (1.3893114, 1.1725779, 0.0).into(),
            (1.4992849, 1.1792196, 0.0).into(),
            (1.5915921, 1.2228394, 0.0).into(),
            (1.5319977, 1.0934557, 0.0).into(),
            (1.6615133, 1.1560599, 0.0).into(),
            (1.7491227, 1.257789, 0.0).into(),
            (1.7821645, 1.3404927, 0.0).into(),
            (1.7934561, 1.4303076, 0.0).into(),
            (1.767753, 1.5651513, 0.0).into(),
            (1.6943312, 1.6765575, 0.0).into(),
            (1.5787218, 1.7523389, 0.0).into(),
            (1.4264561, 1.7803075, 0.0).into(),
            (1.2760342, 1.7521982, 0.0).into(),
            (1.1605811, 1.6761825, 0.0).into(),
            (1.0865655, 1.5647295, 0.0).into(),
            (1.060456, 1.4303076, 0.0).into(),
            (1.0859717, 1.2975732, 0.0).into(),
            (1.1588311, 1.1859325, 0.0).into(),
            (1.273503, 1.1089795, 0.0).into(),
            (1.4244561, 1.0803076, 0.0).into(),
        ],
        indices: vec![
            0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
            13, 14, 14, 15, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 23, 24, 24, 25,
            25, 26, 26, 27, 27, 28, 28, 29, 29, 30, 30, 31, 31, 32, 32, 16, 15, 0,
        ],
    };

    let model_0 = Model {
        indices: &owned_model_0.indices,
        vertices: &owned_model_0.vertices,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(8, result.0.len()); // vertices
    assert_eq!(16, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_simplify_rdp_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("first_index_model_0".to_string(), "0".to_string());
    let _ = config.insert("epsilon".to_string(), "0.5".to_string());
    let _ = config.insert("simplify_3d".to_string(), "false".to_string());
    let _ = config.insert("command".to_string(), "simplify_rdp".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("first_vertex_model_0".to_string(), "0".to_string());

    let owned_model_0 = OwnedModel {
        vertices: vec![
            (-1.8112676, -0.21234381, 0.0).into(),
            (-1.0113943, -0.9753443, 0.0).into(),
            (1.0, -1.0, 0.0).into(),
            (1.5378065, -0.20696306, 0.0).into(),
            (1.0241334, 1.0380125, 0.0).into(),
            (-0.13404018, 1.979902, 0.0).into(),
            (-0.58695304, -1.0762763, 0.04003489).into(),
            (-0.08863782, -0.095835894, 0.04003489).into(),
            (-1.2114286, 0.21341835, 0.04003495).into(),
            (1.2016089, -0.20762604, 0.0400348).into(),
            (0.586953, 1.0762763, 0.04003483).into(),
        ],
        indices: vec![
            0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 0, 7, 8, 8, 10, 7, 6, 6, 9, 10, 7, 7, 9,
        ],
    };

    let model_0 = Model {
        indices: &owned_model_0.indices,
        vertices: &owned_model_0.vertices,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(10, result.0.len()); // vertices
    assert_eq!(22, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_simplify_rdp_3() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("simplify_3d".to_string(), "true".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("command".to_string(), "simplify_rdp".to_string());
    let _ = config.insert("first_vertex_model_0".to_string(), "0".to_string());
    let _ = config.insert("epsilon".to_string(), "0.5".to_string());
    let _ = config.insert("first_index_model_0".to_string(), "0".to_string());

    let owned_model_0 = OwnedModel {
        vertices: vec![
            (0.70696604, 0.655005, 0.04003489).into(),
            (1.4350312, 1.726058, 0.0400348).into(),
            (1.318835, -0.25888658, 0.04003489).into(),
            (0.492464, 1.849285, 0.3837961).into(),
            (-0.46610224, 0.10075432, -0.05803293).into(),
            (-0.033221066, -0.90485096, 0.76515424).into(),
            (1.0233552, -0.57986844, 0.0).into(),
            (2.0278182, 1.1629363, 0.0).into(),
            (1.1876646, 1.5057807, 0.18528388).into(),
            (-1.1262126, 1.6297896, 0.0).into(),
        ],
        indices: vec![
            0, 1, 1, 2, 0, 3, 3, 4, 5, 6, 6, 7, 7, 8, 8, 9, 2, 0, 4, 0, 9, 5,
        ],
    };

    let model_0 = Model {
        indices: &owned_model_0.indices,
        vertices: &owned_model_0.vertices,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(9, result.0.len()); // vertices
    assert_eq!(20, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_divide_into_shapes_1() -> Result<(), HallrError> {
    let indices = vec![
        0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
        14, 14, 15, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 23, 24, 24, 25, 25, 26,
        26, 27, 27, 28, 28, 29, 29, 30, 30, 31, 31, 32, 32, 16, 15, 0,
    ];
    let result = super::divide_into_shapes(&indices);
    assert_eq!(
        result[0],
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]
    );
    assert_eq!(
        result[1],
        vec![16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 16]
    );
    Ok(())
}

#[test]
fn test_divide_into_shapes_2() -> Result<(), HallrError> {
    let indices = vec![1, 2, 2, 5, 0, 2, 5, 4, 4, 3, 0, 3, 1, 6, 6, 7, 7, 3];
    let result = super::divide_into_shapes(&indices);
    assert_eq!(result[0], vec![2, 0, 3]);
    assert_eq!(result[1], vec![2, 5, 4, 3]);
    assert_eq!(result[2], vec![2, 1, 6, 7, 3]);
    Ok(())
}

#[test]
fn test_divide_into_shapes_3() -> Result<(), HallrError> {
    let indices = vec![
        1, 2, 2, 5, 0, 2, 5, 4, 4, 3, 0, 3, 1, 6, 6, 7, 7, 3, 10, 11, 11, 12,
    ];
    let result = super::divide_into_shapes(&indices);
    assert_eq!(result[0], vec![2, 0, 3]);
    assert_eq!(result[1], vec![2, 5, 4, 3]);
    assert_eq!(result[2], vec![2, 1, 6, 7, 3]);
    assert_eq!(result[3], vec![10, 11, 12]);
    Ok(())
}

#[test]
fn test_divide_into_shapes_4() -> Result<(), HallrError> {
    let indices = vec![
        1, 2, 2, 5, 0, 2, 5, 4, 4, 3, 0, 3, 1, 6, 6, 7, 7, 3, 10, 11, 11, 12, 12, 10,
    ];
    let result = super::divide_into_shapes(&indices);
    assert_eq!(result[0], vec![2, 0, 3]);
    assert_eq!(result[1], vec![2, 5, 4, 3]);
    assert_eq!(result[2], vec![2, 1, 6, 7, 3]);
    assert_eq!(result[3], vec![10, 11, 12, 10]);
    Ok(())
}

#[test]
fn test_divide_into_shapes_5() -> Result<(), HallrError> {
    let indices = vec![1, 2, 2, 4, 1, 0, 0, 3, 4, 1, 1, 3];
    let result = super::divide_into_shapes(&indices);
    assert_eq!(result[0], vec![1, 3, 0, 1]);
    assert_eq!(result[1], vec![1, 4, 2, 1]);
    Ok(())
}

#[test]
fn test_divide_into_shapes_6() -> Result<(), HallrError> {
    let indices = vec![1, 2, 2, 4, 1, 0, 0, 3, 4, 1, 1, 3, 11, 12, 12, 13];
    let result = super::divide_into_shapes(&indices);
    assert_eq!(result[0], vec![1, 3, 0, 1]);
    assert_eq!(result[1], vec![1, 4, 2, 1]);
    assert_eq!(result[2], vec![11, 12, 13]);
    Ok(())
}
