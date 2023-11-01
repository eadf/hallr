use crate::prelude::FFIVector3;
use vector_traits::glam::Vec3;
use crate::command::Model;
use crate::command::centerline::process_command;
use crate::command::ConfigType;
use crate::HallrError;

#[cfg(test)]
fn indices() -> Vec<usize> {
    vec![0, 1, 3, 0, 2, 3, 1, 2]
}

#[cfg(test)]
fn vertices() -> Vec<FFIVector3> {
    vec![
        (2.680789, 0.5384059, 0.0).into(),
        (-0.31800875, -2.0773346, 0.0).into(),
        (-1.8870332, -0.39230347, 0.0).into(),
        (-0.4052465, 2.473301, 0.0).into(),
    ]
}

#[cfg(test)]
fn config() -> ConfigType {
    let mut a_map = ConfigType::new();

    let _ = a_map.insert("command".to_string(), "centerline".to_string());
    let _ = a_map.insert("ANGLE".to_string(), "50.0000002530119".to_string());
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
fn test_centerline_1() -> Result<(), HallrError>{
    let indices = indices();
    let vertices = vertices();
    let config = config();
    let models = vec![Model{vertices:&vertices, indices:&indices}];
    let rv = process_command::<Vec3, FFIVector3>(models, config)?;
    println!("rv.vertices: {:?}",rv.0);
    println!("rv.indices: {:?}",rv.1);
    println!("rv.config: {:?}",rv.2);
    Ok(())
}
