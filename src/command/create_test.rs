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
pub(crate) fn process_command(
    config: &ConfigType,
    models: &Vec<Model<'_>>,
) -> Result<(), HallrError> {
    let command = config.get_mandatory_option("command")?;

    println!();
    println!(
        r###"
    use vector_traits::glam::Vec3;
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

    println!("let mut config = ConfigType::default();");
    for (k, v) in config.iter() {
        println!(
            r##"let _= config.insert("{}".to_string(),"{}".to_string());"##,
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
        print!("let models = vec![");
        for i in 0..models.len() {
            print!("owned_model_{}..as_model(), ", i);
        }
        println!("];");
        println!("let result = super::process_command::<Vec3>(config, models)?;");
        //println!("assert_eq!({},result.1.chunks(2).count());", 0);
        println!("assert_eq!({},result.0.len()); // vertices", 0);
        println!("assert_eq!({},result.1.len()); // indices", 0);

        println!("Ok(())");
        println!("}}");
        println!();
    }
    Ok(())
}
