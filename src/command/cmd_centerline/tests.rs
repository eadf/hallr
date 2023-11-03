use crate::{
    command::{cmd_centerline::process_command, ConfigType, Model},
    prelude::FFIVector3,
    HallrError,
};
use vector_traits::glam::Vec3;

#[cfg(test)]
fn indices() -> Vec<usize> {
    vec![0, 3, 1, 0, 2, 1, 2, 3]
}

#[cfg(test)]
fn vertices() -> Vec<FFIVector3> {
    vec![
        (0.0, 0.0, 0.0).into(),
        (0.0, 0.5, 0.0).into(),
        (0.5, 0.5, 0.0).into(),
        (0.5, 0.0, 0.0).into(),
    ]
}

#[cfg(test)]
fn config() -> ConfigType {
    let mut a_map = ConfigType::new();

    let _ = a_map.insert("command".to_string(), "cmd_centerline".to_string());
    let _ = a_map.insert("ANGLE".to_string(), "89".to_string());
    let _ = a_map.insert("SIMPLIFY".to_string(), "true".to_string());
    let _ = a_map.insert("KEEP_INPUT".to_string(), "true".to_string());
    let _ = a_map.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = a_map.insert("NEGATIVE_RADIUS".to_string(), "true".to_string());
    let _ = a_map.insert("REMOVE_INTERNALS".to_string(), "true".to_string());
    let _ = a_map.insert("DISTANCE".to_string(), "0.004999999888241291".to_string());
    let _ = a_map.insert("WELD".to_string(), "true".to_string());
    a_map
}

#[test]
fn test_centerline_1() -> Result<(), HallrError> {
    let indices = indices();
    let vertices = vertices();
    let config = config();
    let models = vec![Model {
        vertices: &vertices,
        indices: &indices,
    }];
    let _rv = process_command::<Vec3>(config, models)?;
    //println!("rv.vertices: {:?}", rv.0);
    //println!("rv.indices: {:?}", rv.1);
    //println!("rv.config: {:?}", rv.2);
    Ok(())
}
