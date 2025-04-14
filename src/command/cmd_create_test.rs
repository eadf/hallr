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
        use ryu;
        ryu::Buffer::new().format::<f32>(*self).to_string()
    }
}

/// This is a command that peeks at incoming data and creates a test case out of it
pub(crate) fn process_command(config: &ConfigType, models: &[Model<'_>]) -> Result<(), HallrError> {
    let command = config.get_mandatory_option("▶")?;

    println!();
    println!(
        r###"
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::command::{{ConfigType, OwnedModel}};
use crate::{{HallrError,command}};
"###
    );
    println!();
    println!(
        r###"#[test]
fn test_{}_1() -> Result<(),HallrError> {{"###,
        command.replace("½", "_5")
    );

    println!("    let mut config = ConfigType::default();");
    for (k, v) in config.iter() {
        if k.eq("CUSTOM_TURTLE") {
            // only use r### for multiline turtles
            println!(
                r####"    let _= config.insert("{}".to_string(),r###"{}"###.to_string());"####,
                k, v
            );
        } else {
            println!(
                r#"    let _= config.insert("{}".to_string(),"{}".to_string());"#,
                k, v
            );
        }
    }
    if !models.is_empty() {
        for (i, model) in models.iter().enumerate() {
            let world_orientation = if model.has_identity_orientation() {
                "OwnedModel::identity_matrix()".to_string()
            } else {
                format!("{:?}", model.copy_world_orientation()?).to_string()
            };
            println!(
                r###"
    let owned_model_{} = OwnedModel{{world_orientation: {}, vertices:vec!["###,
                i, world_orientation
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
    let result = super::process_command(config, models)?;
    command::test_3d_triangulated_mesh(&result);
    assert_eq!(0,result.0.len()); // vertices
    assert_eq!(0,result.1.len()); // indices
    Ok(())
}
"##;
        println!("{}", s);
    }
    Ok(())
}
