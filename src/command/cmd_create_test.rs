// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model, Options};
use crate::HallrError;

trait DecimalRepresentation {
    /// decimal representation will print at least one decimal
    fn dr(&self) -> String;
}

impl DecimalRepresentation for f32 {
    fn dr(&self) -> String {
        if self.fract() == 0.0 {
            format!("{:.1}", self)
        } else {
            format!("{}", self)
        }
    }
}

/// This is a command that peeks at incoming data and creates a test case out of it
#[allow(dead_code)]
pub(crate) fn process_command(config: &ConfigType, models: &[Model<'_>]) -> Result<(), HallrError> {
    let command = config.get_mandatory_option("command")?;

    println!();
    println!(
        r###"
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::command::{{ConfigType, Model, OwnedModel}};
use crate::HallrError;
"###
    );
    println!();
    println!(
        r###"#[test]
fn test_{}_1() -> Result<(),HallrError> {{"###,
        command
    );

    println!("    let mut config = ConfigType::default();");
    for (k, v) in config.iter() {
        println!(
            r####"    let _= config.insert("{}".to_string(),r###"{}"###.to_string());"####,
            k, v
        );
    }
    if !models.is_empty() {
        for (i, model) in models.iter().enumerate() {
            println!(
                r###"
    let owned_model_{} = OwnedModel{{world_orientation: OwnedModel::identity_matrix(), vertices:vec!["###,
                i
            );
            for v in model.vertices.iter() {
                print!("({},{},{}).into(),", v.x.dr(), v.y.dr(), v.z.dr());
            }
            println!("],");
            println!("indices:vec![");
            for i in model.indices.iter() {
                print!("{},", i);
            }
            println!("]}};");
        }
        println!();
        print!("    let models = vec![");
        for i in 0..models.len() {
            print!("owned_model_{}.as_model(), ", i);
        }
        println!("];");
        //println!("assert_eq!({},_result.1.chunks(2).count());", 0);
        let s = r##"
    let _result = super::process_command(config, models)?;
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
    }
    //assert_eq!(0,_result.0.len()); // vertices
    //assert_eq!(0,_result.1.len()); // indices
    Ok(())
}
"##;
        println!("{}", s);
    }
    Ok(())
}
