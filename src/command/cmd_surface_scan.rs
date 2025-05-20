// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model};
use hronn::{
    HronnError, generate_aabb_then_convex_hull, generate_convex_hull_then_aabb,
    prelude::{
        AdaptiveSearchConfig, BallNoseProbe, ConvertTo, MeanderPattern, MeshAnalyzer,
        MeshAnalyzerBuilder, Probe, SearchPattern, SearchPatternConfig, SquareEndProbe,
        TaperedProbe, TriangulatePattern,
    },
};

use crate::{HallrError, command::Options, ffi, prelude::FFIVector3};
use krakel::PointTrait;
use vector_traits::{
    num_traits::AsPrimitive,
    prelude::{GenericVector3, HasXY},
};

#[cfg(test)]
mod tests;
fn do_meander_scan<T>(
    input_config: ConfigType,
    bounding_vertices: &[FFIVector3],
    mesh_analyzer: &MeshAnalyzer<'_, T, FFIVector3>,
    probe: &dyn Probe<T, FFIVector3>,
    minimum_z: T::Scalar,
    step: T::Scalar,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError>
where
    T: GenericVector3,
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    u32: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
    u32: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
{
    let search_config = if input_config.does_option_exist("xy_sample_dist_multiplier")? {
        SearchPatternConfig::<T, FFIVector3>::new(probe, minimum_z).with_adaptive_config(
            AdaptiveSearchConfig::new(
                input_config
                    .get_mandatory_parsed_option::<T::Scalar>("xy_sample_dist_multiplier", None)?
                    * step,
                input_config.get_mandatory_parsed_option::<T::Scalar>(
                    "z_jump_threshold_multiplier",
                    None,
                )? * step,
                input_config.get_mandatory_parsed_option::<bool>("reduce_adaptive", None)?,
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

    let (aabb, convex_hull) = match input_config.get_mandatory_option("bounds")? {
        "CONVEX_HULL" => generate_convex_hull_then_aabb(bounding_vertices),
        "AABB" => generate_aabb_then_convex_hull(bounding_vertices),
        bounds => Err(HronnError::InvalidParameter(format!(
            "{bounds} is not a valid \"bounds\" parameter",
        ))),
    }?;

    let mut results = MeanderPattern::<T, FFIVector3>::new(aabb, convex_hull, step)?
        .search(mesh_analyzer, &search_config)?
        .get_line_data()?;
    let mut return_config = ConfigType::new();

    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::LineWindows.to_string(),
    );
    if let Some(mv) = input_config.get_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }

    let indices = results.lines.pop().unwrap_or_else(Vec::default);

    Ok((results.vertices, indices, return_config))
}

fn do_triangulation_scan<T>(
    input_config: ConfigType,
    bounding_vertices: &[FFIVector3],
    mesh_analyzer: &MeshAnalyzer<'_, T, FFIVector3>,
    probe: &dyn Probe<T, FFIVector3>,
    minimum_z: T::Scalar,
    step: T::Scalar,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError>
where
    T: GenericVector3,
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    u32: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
    u32: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
{
    let (aabb, convex_hull) = match input_config.get_mandatory_option("bounds")? {
        "CONVEX_HULL" => generate_convex_hull_then_aabb(bounding_vertices),
        "AABB" => generate_aabb_then_convex_hull(bounding_vertices),
        bounds => Err(HronnError::InvalidParameter(format!(
            "{bounds} is not a valid \"bounds\" parameter",
        ))),
    }?;

    let search_config = if input_config.does_option_exist("xy_sample_dist_multiplier")? {
        SearchPatternConfig::<T, FFIVector3>::new(probe, minimum_z).with_adaptive_config(
            AdaptiveSearchConfig::new(
                input_config
                    .get_mandatory_parsed_option::<T::Scalar>("xy_sample_dist_multiplier", None)?
                    * step,
                input_config.get_mandatory_parsed_option::<T::Scalar>(
                    "z_jump_threshold_multiplier",
                    None,
                )? * step,
                input_config.get_mandatory_parsed_option::<bool>("reduce_adaptive", None)?,
            ),
        )
    } else {
        SearchPatternConfig::<T, FFIVector3>::new(probe, minimum_z)
    };

    let results = TriangulatePattern::<T, FFIVector3>::new(aabb, convex_hull, step)?
        .search(mesh_analyzer, &search_config)?
        .get_mesh_data()?;
    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    if let Some(mv) = input_config.get_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }
    Ok((results.vertices, results.indices, return_config))
}

pub(crate) fn process_command<T>(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: GenericVector3,
    T::Vector2: PointTrait<PScalar = T::Scalar>,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    u32: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
    u32: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<<FFIVector3 as HasXY>::Scalar>,
    f64: AsPrimitive<T::Scalar>,
{
    if models.len() < 2 {
        Err(HronnError::InvalidParameter(
            "Not enough models detected".to_string(),
        ))?
    }
    let model = &models[0];
    let world_matrix = model.world_orientation.to_vec();
    let bounding_shape = &models[1];

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Triangulated)?;
    input_config.confirm_mesh_packaging(1, ffi::MeshFormat::PointCloud)?;

    let mesh_analyzer = MeshAnalyzerBuilder::<T, FFIVector3>::default()
        .load_from_ref(model.vertices, model.indices)?
        .build()?;
    let bounding_vertices = bounding_shape.vertices;

    let probe_radius = input_config.get_mandatory_parsed_option("probe_radius", None)?;
    let minimum_z = input_config.get_mandatory_parsed_option("minimum_z", None)?;
    let step = input_config.get_mandatory_parsed_option("step", None)?;
    let probe: Box<dyn Probe<T, FFIVector3>> = match input_config.get_mandatory_option("probe")? {
        "SQUARE_END" => Box::new(SquareEndProbe::new(&mesh_analyzer, probe_radius)?),
        "BALL_NOSE" => Box::new(BallNoseProbe::new(&mesh_analyzer, probe_radius)?),
        "TAPERED_END" => {
            let angle = input_config.get_mandatory_parsed_option("probe_angle", None)?;
            Box::new(TaperedProbe::new(&mesh_analyzer, probe_radius, angle)?)
        }
        probe_name => Err(HronnError::InvalidParameter(format!(
            "{probe_name} is not a valid \"probe\" parameter",
        )))?,
    };

    let rv = match input_config.get_mandatory_option("pattern")? {
        "MEANDER" => do_meander_scan::<T>(
            input_config,
            bounding_vertices,
            &mesh_analyzer,
            probe.as_ref(),
            minimum_z,
            step,
        ),
        "TRIANGULATION" => do_triangulation_scan::<T>(
            input_config,
            bounding_vertices,
            &mesh_analyzer,
            probe.as_ref(),
            minimum_z,
            step,
        ),

        pattern => Err(HallrError::InvalidParameter(format!(
            "{pattern} is not a valid option for the \"probe\" parameter",
        ))),
    }?;
    Ok((rv.0, rv.1, world_matrix, rv.2))
}
