//use vector_traits::GenericVector3;
//use crate::ffi::FFIVector3;
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
    println!();
    println!(
        r###"#[test]
fn {}_1() -> Result<(),HallrError> {{"###,
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
        println!(
            r###"
    let owned_model = OwnedModel{{vertices:vec!["###
        );
        let model = &models[0];
        for v in model.vertices.iter() {
            print!("({},{},{}).into(),", v.x.dr(), v.y.dr(), v.z.dr());
        }
        println!("],");
        println!("indices:vec![");
        for i in model.indices.iter() {
            print!("{},", i);
        }
        println!("]}};");
        println!();
        println!(
            "let model = Model{{vertices:&owned_model.vertices, indices:&owned_model.indices}};"
        );
        println!("let result = super::process_command::<Vec3>(config, vec![model])?;");
        println!("assert_eq!({},result.1.chunks(2).count());", 0);
        println!("assert_eq!({},result.1.len();", 0);
        println!("assert_eq!({},result.0.len();", 0);

        println!("Ok(())");
        println!("}}");
        println!();
    }
    Ok(())
}
