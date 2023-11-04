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
