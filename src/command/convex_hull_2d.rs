use super::ConfigType;
use crate::{geo::HashableVector2, /*obj::Obj,*/ prelude::*};
use hronn::prelude::*;
use krakel::PointTrait;
use linestring::linestring_2d::convex_hull;
use vector_traits::{approx::UlpsEq, GenericScalar, GenericVector2, GenericVector3, HasXYZ};

pub(crate) fn process_command<T: GenericVector3, MESH: HasXYZ>(
    vertices: &[MESH],
    _indices: &[usize],
    _config: ConfigType,
) -> Result<(Vec<MESH>, Vec<usize>, ConfigType), HallrError>
where
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T::Scalar: UlpsEq,
    T: ConvertTo<MESH>,
    MESH: ConvertTo<T>,
    HashableVector2: From<T::Vector2>,
{
    // convert the input vertices to 2d point cloud
    let input: Vec<_> = vertices.iter().map(|v| v.to().to_2d()).collect();
    let mut obj = Obj::<MESH>::new("convex_hull");
    // calculate the convex hull, and convert back to 3d MESH vertices
    convex_hull::graham_scan(&input)
        .points()
        .iter()
        .for_each(|v| obj.continue_line(v.to_3d(T::Scalar::ZERO).to()));
    let mut config = ConfigType::new();
    let _ = config.insert("mesh.format".to_string(), "line".to_string());
    println!(
        "convex_hull_2d operation returning {} vertices",
        obj.vertices.len()
    );
    Ok((obj.vertices, obj.lines.pop().unwrap_or_default(), config))
}