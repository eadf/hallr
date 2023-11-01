use super::{ConfigType, Options};
use crate::{geo::HashableVector2, prelude::*};
use hronn::prelude::*;

use krakel::PointTrait;
use linestring::linestring_2d::{convex_hull, Aabb2, LineString2};
use vector_traits::{num_traits::AsPrimitive, GenericVector3, HasXYZ};

fn aabb_delaunay_triangulation_2d<T: GenericVector3, MESH: HasXYZ>(
    _config: ConfigType,
    object_vertices: &[MESH],
    _object_indices: &[usize],
    bounding_vertices: &[MESH],
    _bounding_indices: &[usize],
) -> Result<(Vec<MESH>, Vec<usize>, ConfigType), HallrError>
where
    T: ConvertTo<MESH>,
    MESH: ConvertTo<T>,
    T::Scalar: AsPrimitive<MESH::Scalar>,
{
    if bounding_vertices.is_empty() {
        return Err(HallrError::NoData("The bounding box is empty".to_string()));
    }
    // compute the AABB of the bounding_vertices regardless of interconnection
    let aabb = {
        let mut aabb = Aabb2::<T::Vector2>::default();
        for v in bounding_vertices {
            aabb.update_with_point(v.to().to_2d());
        }
        aabb
    };
    // Use the AABB to generate a convex hull
    let hull: Vec<T::Vector2> = aabb
        .convex_hull::<T::Vector2>()
        .unwrap_or(Vec::<T::Vector2>::default())
        .into_iter()
        //.map(|v| v.to_3d(T::Scalar::ZERO).to())
        .collect();

    let results = triangulate_vertices::<T, MESH>(aabb, &hull, object_vertices)?;
    let mut return_config = ConfigType::new();
    let _ = return_config.insert("mesh.format".to_string(), "triangulated".to_string());
    Ok((results.0, results.1, return_config))
}

fn convex_hull_delaunay_triangulation_2d<T: GenericVector3, MESH: HasXYZ>(
    _config: ConfigType,
    object_vertices: &[MESH],
    _object_indices: &[usize],
    bounding_vertices: &[MESH],
    bounding_indices: &[usize],
) -> Result<(Vec<MESH>, Vec<usize>, ConfigType), HallrError>
where
    T: ConvertTo<MESH>,
    MESH: ConvertTo<T>,
    T::Scalar: AsPrimitive<MESH::Scalar>,
{
    if bounding_vertices.is_empty() {
        return Err(HallrError::NoData("The bounding box is empty".to_string()));
    }

    // do not limit us to a line bound, - yet
    //let bounding_indices =
    //    crate::collision::continuous_loop_from_unordered_edges(bounding_indices)?;
    println!("bounding_indices {:?}", bounding_indices.len());
    println!("bounding_vertices {:?}", bounding_vertices.len());

    let convex_hull: LineString2<T::Vector2> = {
        // strip the Z coordinate off the bounding shape
        let point_cloud = LineString2::<T::Vector2>::with_iter(
            bounding_indices
                .iter()
                .map(|i| bounding_vertices[*i].to().to_2d()),
        );
        convex_hull::graham_scan(&point_cloud.0)
    };
    let aabb = Aabb2::with_points(&convex_hull.0);

    let results = triangulate_vertices::<T, MESH>(aabb, &convex_hull.0, object_vertices)?;
    let mut return_config = ConfigType::new();
    let _ = return_config.insert("mesh.format".to_string(), "triangulated".to_string());
    Ok((results.0, results.1, return_config))
}

pub(crate) fn process_command<T: GenericVector3, MESH: HasXYZ>(
    vertices: &[MESH],
    indices: &[usize],
    config: ConfigType,
) -> Result<(Vec<MESH>, Vec<usize>, ConfigType), HallrError>
where
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<MESH>,
    MESH: ConvertTo<T>,
    T::Scalar: AsPrimitive<MESH::Scalar>,
    HashableVector2: From<T::Vector2>,
{
    let start_vertex_index_for_bounding: usize =
        config.get_mandatory_parsed_option("start_vertex_index_for_bounding")?;
    let start_index_for_bounding: usize =
        config.get_mandatory_parsed_option("start_index_for_bounding")?;

    let object_vertices = &vertices[0..start_vertex_index_for_bounding];
    let object_indices = &indices[0..start_index_for_bounding];

    let bounding_indices = &indices[start_index_for_bounding..];
    let bounding_vertices = &vertices[start_vertex_index_for_bounding..];

    match config.get_mandatory_option("bounds")? {
        "CONVEX_HULL" => convex_hull_delaunay_triangulation_2d::<T, MESH>(
            config,
            object_vertices,
            object_indices,
            bounding_vertices,
            bounding_indices,
        ),
        "AABB" => aabb_delaunay_triangulation_2d::<T, MESH>(
            config,
            object_vertices,
            object_indices,
            bounding_vertices,
            bounding_indices,
        ),
        bounds => Err(HallrError::InvalidParameter(format!(
            "{} is not a valid \"bounds\" parameter",
            bounds
        ))),
    }
}
