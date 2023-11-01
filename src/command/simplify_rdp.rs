use super::{ConfigType, Options};
use crate::{
    geo::{reconstruct_from_unordered_edges, HashableVector2},
    prelude::*,
};
use hronn::prelude::*;
use linestring::linestring_2d::LineString2;
use vector_traits::{GenericScalar, GenericVector2, GenericVector3, HasXYZ};

pub(crate) fn process_command<T: GenericVector3, MESH: HasXYZ>(
    vertices: &[MESH],
    indices: &[usize],
    config: ConfigType,
) -> Result<(Vec<MESH>, Vec<usize>, ConfigType), HallrError>
where
    T: ConvertTo<MESH>,
    MESH: ConvertTo<T>,
    HashableVector2: From<T::Vector2>,
{
    let epsilon: T::Scalar = config.get_mandatory_parsed_option("epsilon")?;
    let mut obj = Obj::<MESH>::new("simplified_rdp");
    //println!("rust: vertices.len():{}", vertices.len());
    //println!("rust: indices.len():{}", indices.len());
    //println!("rust: indices:{:?}", indices);
    if vertices.len() > 1 {
        // convert the input vertices to 2d point cloud
        let vertices: Vec<T::Vector2> = vertices.iter().map(|v| v.to().to_2d()).collect();
        //println!("Vertices:{:?}", vertices);
        //println!("Indices:{:?}", indices);
        let indices = reconstruct_from_unordered_edges(indices)?;
        //println!("sorted indices:{:?}", indices);
        let line: LineString2<T::Vector2> = indices.into_iter().map(|i| vertices[i]).collect();
        //println!("Vertices sorted:{:?}", line.points);
        let line = line.simplify_rdp(epsilon);
        line.0
            .iter()
            .for_each(|v: &T::Vector2| obj.continue_line(v.to_3d(T::Scalar::ZERO).to()));
        //println!("result edges before close:{:?}", obj.lines.first());
        if line.is_connected() {
            obj.close_line();
        }
    }
    //println!("result vertices:{:?}", obj.vertices);
    //println!("result edges:{:?}", obj.lines.first());
    let mut config = ConfigType::new();
    let _ = config.insert("mesh.format".to_string(), "line".to_string());
    println!(
        "simplify_rdp operation returning {} vertices, {} edges",
        obj.vertices.len(),
        obj.lines.first().map(|v| v.len()).unwrap_or(0)
    );
    Ok((obj.vertices, obj.lines.pop().unwrap_or_default(), config))
}
