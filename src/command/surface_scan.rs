use super::{get_mandatory_numeric_option, get_mandatory_option, ConfigType};
use hronn::prelude::*;

use crate::{
    command::{does_option_exist, get_mandatory_bool_option},
    geo::HashableVector2,
    HallrError,
};
use krakel::PointTrait;
use vector_traits::{num_traits::AsPrimitive, Approx, GenericVector3, HasXYZ};

fn do_meander_scan<T: GenericVector3, MESH: HasXYZ>(
    config: ConfigType,
    bounding_vertices: &[MESH],
    _bounding_indices: &[usize],
    mesh_analyzer: &MeshAnalyzer<'_, T, MESH>,
    probe: &dyn Probe<T, MESH>,
    minimum_z: T::Scalar,
    step: T::Scalar,
) -> Result<(Vec<MESH>, Vec<usize>, ConfigType), HallrError>
where
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<MESH>,
    MESH: ConvertTo<T> + Approx,
    u32: AsPrimitive<MESH::Scalar>,
    u32: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<MESH::Scalar>,
{
    let search_config = if does_option_exist("xy_sample_dist_multiplier", &config)? {
        SearchPatternConfig::<T, MESH>::new(probe, minimum_z).with_adaptive_config(
            AdaptiveSearchConfig::new(
                get_mandatory_numeric_option::<T::Scalar>("xy_sample_dist_multiplier", &config)?
                    * step,
                get_mandatory_numeric_option::<T::Scalar>("z_jump_threshold_multiplier", &config)?
                    * step,
                get_mandatory_bool_option("reduce_adaptive", &config)?,
            ),
        )
    } else {
        SearchPatternConfig::<T, MESH>::new(probe, minimum_z)
    };

    // do not limit us to a line bound, - yet
    //let bounding_indices =
    //    crate::hronn::continuous_loop_from_unordered_edges(bounding_indices)?;
    //println!("bounding_indices {:?}", bounding_indices.len());
    //println!("bounding_vertices {:?}", bounding_vertices.len());

    let (aabb, convex_hull) = match get_mandatory_option("bounds", &config)? {
        "CONVEX_HULL" => generate_convex_hull_then_aabb(bounding_vertices),
        "AABB" => generate_aabb_then_convex_hull(bounding_vertices),
        bounds => Err(HronnError::InvalidParameter(format!(
            "{} is not a valid \"bounds\" parameter",
            bounds
        ))),
    }?;

    let mut results = MeanderPattern::<T, MESH>::new(aabb, convex_hull, step)?
        .search(mesh_analyzer, &search_config)?
        .get_line_data()?;
    let mut return_config = ConfigType::new();

    let _ = return_config.insert("mesh.format".to_string(), "line".to_string());

    let indices = results.lines.pop().unwrap_or_else(Vec::default);

    Ok((results.vertices, indices, return_config))
}

fn do_triangulation_scan<T: GenericVector3, MESH: HasXYZ>(
    config: ConfigType,
    bounding_vertices: &[MESH],
    _bounding_indices: &[usize],
    mesh_analyzer: &MeshAnalyzer<'_, T, MESH>,
    probe: &dyn Probe<T, MESH>,
    minimum_z: T::Scalar,
    step: T::Scalar,
) -> Result<(Vec<MESH>, Vec<usize>, ConfigType), HallrError>
where
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<MESH>,
    MESH: ConvertTo<T> + Approx,
    u32: AsPrimitive<MESH::Scalar>,
    u32: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<MESH::Scalar>,
{
    let (aabb, convex_hull) = match get_mandatory_option("bounds", &config)? {
        "CONVEX_HULL" => generate_convex_hull_then_aabb(bounding_vertices),
        "AABB" => generate_aabb_then_convex_hull(bounding_vertices),
        bounds => Err(HronnError::InvalidParameter(format!(
            "{} is not a valid \"bounds\" parameter",
            bounds
        ))),
    }?;

    let search_config = if does_option_exist("xy_sample_dist_multiplier", &config)? {
        SearchPatternConfig::<T, MESH>::new(probe, minimum_z).with_adaptive_config(
            AdaptiveSearchConfig::new(
                get_mandatory_numeric_option::<T::Scalar>("xy_sample_dist_multiplier", &config)?
                    * step,
                get_mandatory_numeric_option::<T::Scalar>("z_jump_threshold_multiplier", &config)?
                    * step,
                get_mandatory_bool_option("reduce_adaptive", &config)?,
            ),
        )
    } else {
        SearchPatternConfig::<T, MESH>::new(probe, minimum_z)
    };

    let results = TriangulatePattern::<T, MESH>::new(aabb, convex_hull, step)?
        .search(mesh_analyzer, &search_config)?
        .get_mesh_data()?;
    let mut return_config = ConfigType::new();
    let _ = return_config.insert("mesh.format".to_string(), "triangulated".to_string());
    Ok((results.vertices, results.indices, return_config))
}

pub(crate) fn process_command<T: GenericVector3, MESH: HasXYZ>(
    vertices: &[MESH],
    indices: &[usize],
    config: ConfigType,
) -> Result<(Vec<MESH>, Vec<usize>, ConfigType), HallrError>
where
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<MESH>,
    MESH: ConvertTo<T> + Approx,
    u32: AsPrimitive<MESH::Scalar>,
    u32: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<MESH::Scalar>,
    HashableVector2: From<T::Vector2>,
{
    let start_vertex_index_for_bounding: usize =
        get_mandatory_numeric_option("start_vertex_index_for_bounding", &config)?;
    let start_index_for_bounding: usize =
        get_mandatory_numeric_option("start_index_for_bounding", &config)?;

    let mesh_analyzer = MeshAnalyzerBuilder::<T, MESH>::default()
        .load_from_ref(
            &vertices[0..start_vertex_index_for_bounding],
            &indices[0..start_index_for_bounding],
        )?
        .build()?;
    let bounding_indices = &indices[start_index_for_bounding..];
    let bounding_vertices = &vertices[start_vertex_index_for_bounding..];

    let probe_radius = get_mandatory_numeric_option("probe_radius", &config)?;
    let minimum_z = get_mandatory_numeric_option("minimum_z", &config)?;
    let step = get_mandatory_numeric_option("step", &config)?;
    let probe: Box<dyn Probe<T, MESH>> = match get_mandatory_option("probe", &config)? {
        "SQUARE_END" => Box::new(SquareEndProbe::new(&mesh_analyzer, probe_radius)?),
        "BALL_NOSE" => Box::new(BallNoseProbe::new(&mesh_analyzer, probe_radius)?),
        probe_name => Err(HronnError::InvalidParameter(format!(
            "{} is not a valid \"probe\" parameter",
            probe_name
        )))?,
    };

    match get_mandatory_option("pattern", &config)? {
        "MEANDER" => do_meander_scan::<T, MESH>(
            config,
            bounding_vertices,
            bounding_indices,
            &mesh_analyzer,
            probe.as_ref(),
            minimum_z,
            step,
        ),
        "TRIANGULATION" => do_triangulation_scan::<T, MESH>(
            config,
            bounding_vertices,
            bounding_indices,
            &mesh_analyzer,
            probe.as_ref(),
            minimum_z,
            step,
        ),

        pattern => Err(HallrError::InvalidParameter(format!(
            "{} is not a valid option for the \"probe\" parameter",
            pattern
        ))),
    }
}