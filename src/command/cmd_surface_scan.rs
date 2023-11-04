use super::{ConfigType, Model};
use hronn::{
    generate_aabb_then_convex_hull, generate_convex_hull_then_aabb,
    prelude::{
        AdaptiveSearchConfig, BallNoseProbe, ConvertTo, MeanderPattern, MeshAnalyzer,
        MeshAnalyzerBuilder, Probe, SearchPattern, SearchPatternConfig, SquareEndProbe,
        TriangulatePattern,
    },
    HronnError,
};

use crate::{command::Options, prelude::FFIVector3, utils::HashableVector2, HallrError};
use krakel::PointTrait;
use vector_traits::{num_traits::AsPrimitive, Approx, GenericVector3, HasXY};

#[cfg(test)]
mod tests;
fn do_meander_scan<T: GenericVector3>(
    config: ConfigType,
    bounding_vertices: &[FFIVector3],
    _bounding_indices: &[usize],
    mesh_analyzer: &MeshAnalyzer<'_, T, FFIVector3>,
    probe: &dyn Probe<T, FFIVector3>,
    minimum_z: T::Scalar,
    step: T::Scalar,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError>
where
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T> + Approx,
    u32: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
    u32: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
{
    let search_config = if config.does_option_exist("xy_sample_dist_multiplier")? {
        SearchPatternConfig::<T, FFIVector3>::new(probe, minimum_z).with_adaptive_config(
            AdaptiveSearchConfig::new(
                config.get_mandatory_parsed_option::<T::Scalar>("xy_sample_dist_multiplier")?
                    * step,
                config.get_mandatory_parsed_option::<T::Scalar>("z_jump_threshold_multiplier")?
                    * step,
                config.get_mandatory_parsed_option::<bool>("reduce_adaptive")?,
            ),
        )
    } else {
        SearchPatternConfig::<T, FFIVector3>::new(probe, minimum_z)
    };

    // do not limit us to a line bound, - yet
    //let bounding_indices =
    //    crate::hronn::continuous_loop_from_unordered_edges(bounding_indices)?;
    //println!("bounding_indices {:?}", bounding_indices.len());
    //println!("bounding_vertices {:?}", bounding_vertices.len());

    let (aabb, convex_hull) = match config.get_mandatory_option("bounds")? {
        "CONVEX_HULL" => generate_convex_hull_then_aabb(bounding_vertices),
        "AABB" => generate_aabb_then_convex_hull(bounding_vertices),
        bounds => Err(HronnError::InvalidParameter(format!(
            "{} is not a valid \"bounds\" parameter",
            bounds
        ))),
    }?;

    let mut results = MeanderPattern::<T, FFIVector3>::new(aabb, convex_hull, step)?
        .search(mesh_analyzer, &search_config)?
        .get_line_data()?;
    let mut return_config = ConfigType::new();

    let _ = return_config.insert("mesh.format".to_string(), "line".to_string());

    let indices = results.lines.pop().unwrap_or_else(Vec::default);

    Ok((results.vertices, indices, return_config))
}

fn do_triangulation_scan<T: GenericVector3>(
    config: ConfigType,
    bounding_vertices: &[FFIVector3],
    _bounding_indices: &[usize],
    mesh_analyzer: &MeshAnalyzer<'_, T, FFIVector3>,
    probe: &dyn Probe<T, FFIVector3>,
    minimum_z: T::Scalar,
    step: T::Scalar,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError>
where
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T> + Approx,
    u32: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
    u32: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
{
    let (aabb, convex_hull) = match config.get_mandatory_option("bounds")? {
        "CONVEX_HULL" => generate_convex_hull_then_aabb(bounding_vertices),
        "AABB" => generate_aabb_then_convex_hull(bounding_vertices),
        bounds => Err(HronnError::InvalidParameter(format!(
            "{} is not a valid \"bounds\" parameter",
            bounds
        ))),
    }?;

    let search_config = if config.does_option_exist("xy_sample_dist_multiplier")? {
        SearchPatternConfig::<T, FFIVector3>::new(probe, minimum_z).with_adaptive_config(
            AdaptiveSearchConfig::new(
                config.get_mandatory_parsed_option::<T::Scalar>("xy_sample_dist_multiplier")?
                    * step,
                config.get_mandatory_parsed_option::<T::Scalar>("z_jump_threshold_multiplier")?
                    * step,
                config.get_mandatory_parsed_option::<bool>("reduce_adaptive")?,
            ),
        )
    } else {
        SearchPatternConfig::<T, FFIVector3>::new(probe, minimum_z)
    };

    let results = TriangulatePattern::<T, FFIVector3>::new(aabb, convex_hull, step)?
        .search(mesh_analyzer, &search_config)?
        .get_mesh_data()?;
    let mut return_config = ConfigType::new();
    let _ = return_config.insert("mesh.format".to_string(), "triangulated".to_string());
    Ok((results.vertices, results.indices, return_config))
}

pub(crate) fn process_command<T: GenericVector3>(
    config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError>
where
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T> + Approx,
    u32: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
    u32: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
    HashableVector2: From<T::Vector2>,
{
    if models.len() < 2 {
        Err(HronnError::InvalidParameter(
            "Not enough models detected".to_string(),
        ))?
    }
    let model = &models[0];
    let bounding_shape = &models[1];

    let mesh_analyzer = MeshAnalyzerBuilder::<T, FFIVector3>::default()
        .load_from_ref(model.vertices, model.indices)?
        .build()?;
    let bounding_indices = bounding_shape.indices;
    let bounding_vertices = bounding_shape.vertices;

    let probe_radius = config.get_mandatory_parsed_option("probe_radius")?;
    let minimum_z = config.get_mandatory_parsed_option("minimum_z")?;
    let step = config.get_mandatory_parsed_option("step")?;
    let probe: Box<dyn Probe<T, FFIVector3>> = match config.get_mandatory_option("probe")? {
        "SQUARE_END" => Box::new(SquareEndProbe::new(&mesh_analyzer, probe_radius)?),
        "BALL_NOSE" => Box::new(BallNoseProbe::new(&mesh_analyzer, probe_radius)?),
        probe_name => Err(HronnError::InvalidParameter(format!(
            "{} is not a valid \"probe\" parameter",
            probe_name
        )))?,
    };

    match config.get_mandatory_option("pattern")? {
        "MEANDER" => do_meander_scan::<T>(
            config,
            bounding_vertices,
            bounding_indices,
            &mesh_analyzer,
            probe.as_ref(),
            minimum_z,
            step,
        ),
        "TRIANGULATION" => do_triangulation_scan::<T>(
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
