use crate::{
    command::{ConfigType, OwnedModel},
    HallrError,
};

#[test]
fn test_sdf_mesh_2_5_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("SDF_DIVISIONS".to_string(), "20".to_string());
    let _ = config.insert("command".to_string(), "sdf_mesh_2_5".to_string());
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![
            (-1.0, -1.0, 0.0).into(),
            (0.014304634, 0.021932945, 0.63773185).into(),
            (0.014304634, 0.021932945, 0.6377318).into(),
            (-0.48725998, 0.53284, 0.0).into(),
            (0.11475183, 0.05492184, 0.6363602).into(),
            (1.0, 1.0, 0.0).into(),
            (0.11475183, 0.05492184, 0.6363603).into(),
            (0.65058, -0.43409, 0.0).into(),
        ],
        indices: vec![0, 1, 2, 3, 1, 4, 4, 5, 6, 7],
    };

    let models = vec![owned_model_0.as_model()];
    let result = super::process_command(config, models)?;
    assert_eq!(1279, result.0.len()); // vertices
    assert_eq!(6384, result.1.len()); // indices
    Ok(())
}
