use crate::{
    command::{ConfigType, Model, OwnedModel},
    HallrError,
};
use vector_traits::glam::Vec3;

#[test]
fn test_voronoi_mesh_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("command".to_string(), "voronoi_mesh".to_string());
    let _ = config.insert("DISTANCE".to_string(), "0.2864788911621093".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("first_index_model_0".to_string(), "0".to_string());

    let owned_model_0 = OwnedModel {
        vertices: vec![
            (-1.3491066, -0.42415974, 0.0).into(),
            (0.42415974, -1.3491066, 0.0).into(),
            (-0.42415974, 1.3491066, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 3, 2],
    };

    let model_0 = Model {
        indices: &owned_model_0.indices,
        vertices: &owned_model_0.vertices,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(5, result.0.len()); // vertices
    assert_eq!(12, result.1.len()); // indices
    assert_eq!("triangulated", result.2.get("mesh.format").unwrap());
    Ok(())
}

#[test]
fn test_voronoi_mesh_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("DISTANCE".to_string(), "0.2864788911621093".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("command".to_string(), "voronoi_mesh".to_string());

    let owned_model_0 = OwnedModel {
        vertices: vec![
            (-1.3491066, -0.42415974, 0.0).into(),
            (0.42415974, -1.3491066, 0.0).into(),
            (-0.420259, 1.3558924, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
            (1.1850299, 1.4086196, 0.0).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 3, 2, 3, 4, 4, 5, 5, 2],
    };

    let model_0 = Model {
        indices: &owned_model_0.indices,
        vertices: &owned_model_0.vertices,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(8, result.0.len()); // vertices
    assert_eq!(27, result.1.len()); // indices
    Ok(())
}

#[test]
fn test_voronoi_mesh_3() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("DISTANCE".to_string(), "0.2864788911621093".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("command".to_string(), "voronoi_mesh".to_string());

    let owned_model_0 = OwnedModel {
        vertices: vec![
            (-1.3491066, -0.42415974, 0.0).into(),
            (0.42415974, -1.3491066, 0.0).into(),
            (-0.420259, 1.3558924, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
            (1.3491066, 0.42415974, 0.0).into(),
            (-0.018198848, 0.30930626, 0.0).into(),
        ],
        indices: vec![2, 0, 0, 1, 1, 3, 3, 2, 3, 4],
    };

    let model_0 = Model {
        indices: &owned_model_0.indices,
        vertices: &owned_model_0.vertices,
    };
    let models = vec![model_0];
    let result = super::process_command::<Vec3>(config, models)?;
    assert_eq!(25, result.0.len()); // vertices
    assert_eq!(132, result.1.len()); // indices
    Ok(())
}
