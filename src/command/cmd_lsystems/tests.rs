// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{ConfigType, OwnedModel},
};

#[test]
fn test_lsystems_1() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("command".to_string(), r###"lsystems"###.to_string());
    let _ = config.insert(
        "🐢".to_string(),
        r###"
token("X", Turtle::Nop))
token("F", Turtle::GeodesicForward(1.0)) # step forward along the surface
token("→", Turtle::GeodesicYaw(120)) # turn left on the surface
axiom("F X")
rule("X", "→ F X → F X")
geodesic_radius(5.0)
iterations(5)
"###
        .to_string(),
    );

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![],
        indices: vec![],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;
    println!("{:?}", _result.0);
    println!("{:?}", _result.1);

    /*assert_eq!(_result.1.len() % 3, 0);
    assert!(!_result.1.is_empty());
    let number_of_vertices = _result.0.len();
    assert!(number_of_vertices>0);

    for t in _result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }*/
    //assert_eq!(0,_result.0.len()); // vertices
    //assert_eq!(0,_result.1.len()); // indices
    Ok(())
}

#[test]
fn test_lsystems_8() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        "🐢".to_string(),
        r###"token("X", Turtle::Nop)
token("F", Turtle::Forward(10.0))
token("^", Turtle::Pitch(90.0))
token("&", Turtle::Pitch(-90.0))
token("+", Turtle::Yaw(90.0))
token("-", Turtle::Yaw(-90.0))
token(">", Turtle::Roll(90.0))
token("<", Turtle::Roll(-90.0))
round()
axiom("X")
rule("X", "^<XF^<XFX-F^>>XFX&F+>>XFX-F>X->")
iterations(1)"###
            .to_string(),
    );
    let _ = config.insert("command".to_string(), r###"lsystems"###.to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![],
        indices: vec![],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;
    /*assert_eq!(_result.1.len() % 3, 0);
    assert!(!_result.1.is_empty());
    let number_of_vertices = _result.0.len();
    assert!(number_of_vertices>0);

    for t in _result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }*/
    //assert_eq!(0,_result.0.len()); // vertices
    //assert_eq!(0,_result.1.len()); // indices
    Ok(())
}

#[test]
fn test_lsystems_7() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        "🐢".to_string(),
        r###"
token("A", Turtle::Nop)
token("B", Turtle::Nop)
token("C", Turtle::Nop)
token("D", Turtle::Nop)
token("F", Turtle::Forward(10.0))
token("+", Turtle::Yaw(90.0))
token("-", Turtle::Yaw(-90.0))
token("&", Turtle::Pitch(90.0))
token("∧", Turtle::Pitch(-90.0))
token("\", Turtle::Roll(90.0))
token("/", Turtle::Roll(-90.0))
token("|", Turtle::Yaw(180.0))
axiom("A")
rule("A", " B-F+CFC+F-D&F∧D-F+&&CFC+F+B//")
rule("B", " A&F∧CFB∧F∧D∧∧-F-D∧|F∧B|FC∧F∧A//")
rule("C", " |D∧|F∧B-F+C∧F∧A&&FA&F∧C+F+B∧F∧D//")
rule("D", " |CFB-F+B|FA&F∧A&&FB-F+B|FC//")
iterations(3)
timeout(2)
"###
        .to_string(),
    );
    let _ = config.insert("command".to_string(), r###"lsystems"###.to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![],
        indices: vec![],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;
    /*assert_eq!(_result.1.len() % 3, 0);
    assert!(!_result.1.is_empty());
    let number_of_vertices = _result.0.len();
    assert!(number_of_vertices>0);

    for t in _result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }*/
    //assert_eq!(0,_result.0.len()); // vertices
    //assert_eq!(0,_result.1.len()); // indices
    Ok(())
}

#[test]
fn test_lsystems_6() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("command".to_string(), r###"lsystems"###.to_string());
    let _ = config.insert(
        "🐢".to_string(),
        r###"
# https://en.wikipedia.org/wiki/L-system#Examples_of_L-systems
# build fractal binary tree
token("0", Turtle::Forward(1.0))
token("1", Turtle::Forward(1.0))
token("L", Turtle::Yaw(45.0))
token("R", Turtle::Yaw(-45.0))
token("[", Turtle::Push)
token("]", Turtle::Pop)
axiom("0")
rule("1", " 11")
rule("0", " 1[L0]R0")
rotate(90.0, 0.0, 0.0)
iterations(2)
timeout(2)
"###
        .to_string(),
    );

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![],
        indices: vec![],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;

    //println!("v:{:?}", _result.0);
    //println!();
    //for p in _result.0.chunks_exact(2) {
    //    println!("{:?}-{:?}", p[0], p[1]);
    //}
    //println!("i:{:?}", _result.1);
    //println!();
    //for p in _result.1.chunks_exact(2) {
    //    println!("{:?}-{:?}", _result.0[p[0]], _result.0[p[1]]);
    //}

    /*assert_eq!(_result.1.len() % 3, 0);
    assert!(!_result.1.is_empty());
    let number_of_vertices = _result.0.len();
    assert!(number_of_vertices>0);

    for t in _result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }*/
    //assert_eq!(0,_result.0.len()); // vertices
    //assert_eq!(0,_result.1.len()); // indices
    Ok(())
}

#[test]
fn test_lsystems_2() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert("command".to_string(), r###"lsystems"###.to_string());
    let _ = config.insert(
        "🐢".to_string(),
        r###"
# build a koch curve in 3d
token("-", Turtle::Rotate(40.0, -90.0, 0.0))
token("&", Turtle::Forward(30.0))
token("?", Turtle::Forward(30.0))
token("+", Turtle::Pitch(90.0))
axiom("?")
rule("?", " ? + ? - ? & ? + ?")
iterations(3)
timeout(1)
"###
        .to_string(),
    );

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![],
        indices: vec![],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;
    /*
    assert_eq!(_result.1.len() % 3, 0);
    assert!(!_result.1.is_empty());
    let number_of_vertices = _result.0.len();
    assert!(number_of_vertices>0);

    for t in _result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }*/
    //assert_eq!(0,_result.0.len()); // vertices
    //assert_eq!(0,_result.1.len()); // indices
    Ok(())
}

#[test]
fn test_lsystems_3() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        "🐢".to_string(),
        r###"
token("0", Turtle::Forward(1.0)))
token("1", Turtle::Forward(1.0)))
token("L", Turtle::Yaw(45.0)))
token("R", Turtle::Yaw(-45.0)))
token("[", Turtle::Push)
token("]", Turtle::Pop)
axiom("0")
rule("1", " 11")
rule("0", " 1[L0]R0")
rotate(90.0, 0.0, 0.0)
iterations(2)
timeout(1)
"###
        .to_string(),
    );
    let _ = config.insert("command".to_string(), r###"lsystems"###.to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![],
        indices: vec![],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;

    /*assert_eq!(_result.1.len() % 3, 0);
    assert!(!_result.1.is_empty());
    let number_of_vertices = _result.0.len();
    assert!(number_of_vertices>0);

    for t in _result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }*/
    //assert_eq!(0,_result.0.len()); // vertices
    //assert_eq!(0,_result.1.len()); // indices
    Ok(())
}

#[test]
fn test_lsystems_4() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        "🐢".to_string(),
        r###"
token("0", Turtle::Forward(50.0)))
token("1", Turtle::Forward(50.0)))
token("L", Turtle::Yaw(45.0)))
token("R", Turtle::Yaw(-45.0)))
token("[", Turtle::Push)
token("]", Turtle::Pop)
axiom("0")
rule("1", " 11")
rule("0", " 1[L0]R0")
rotate(90.0, 0.0, 0.0)
iterations(3)
timeout(1)
"###
        .to_string(),
    );
    let _ = config.insert("command".to_string(), r###"lsystems"###.to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![],
        indices: vec![],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;
    /*
    assert_eq!(result.1.len() % 3, 0);
    assert!(!result.1.is_empty());
    let number_of_vertices = result.0.len();
    assert!(number_of_vertices>0);

    for t in result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }
    //assert_eq!(0,result.0.len()); // vertices
    //assert_eq!(0,result.1.len()); // indices
     */
    Ok(())
}

#[test]
fn test_lsystems_5() -> Result<(), HallrError> {
    let mut config = ConfigType::default();
    let _ = config.insert(
        "🐢".to_string(),
        r#"
    token("X", Turtle::Nop)
    token("Y", Turtle::Nop)
    token("F", Turtle::Forward(1))
    token("+", Turtle::Yaw(-90))
    token("-", Turtle::Pitch(90))
    axiom("F X")
    rule("X","X + Y F +")
    rule("Y","- F X - Y")
    round()
    iterations(4)
    timeout(1)
    "#
        .to_string(),
    );
    let _ = config.insert("command".to_string(), "lsystems".to_string());

    let owned_model_0 = OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices: vec![],
        indices: vec![],
    };

    let models = vec![owned_model_0.as_model()];

    let _result = super::process_command(config, models)?;
    println!("result:{:?}", _result);
    /*
    assert_eq!(result.1.len() % 3, 0);
    assert!(!result.1.is_empty());
    let number_of_vertices = result.0.len();
    assert!(number_of_vertices > 0);

    for t in result.1.chunks_exact(3) {
        assert_ne!(t[0], t[1]);
        assert_ne!(t[0], t[2]);
        assert_ne!(t[1], t[2]);

        assert!(
            t[0] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[1] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        );
        assert!(
            t[2] < number_of_vertices,
            "{:?} >= {}",
            t[2],
            number_of_vertices
        )
    }*/
    //assert_eq!(0,result.0.len()); // vertices
    //assert_eq!(0,result.1.len()); // indices
    Ok(())
}
