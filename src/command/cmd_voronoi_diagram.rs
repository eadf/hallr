// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    command::{ConfigType, Model, Options, OwnedModel},
    ffi::FFIVector3,
    utils::{voronoi_utils, GrowingVob},
    HallrError,
};
use boostvoronoi as BV;
use centerline::{HasMatrix4, Matrix4};
use hronn::prelude::ConvertTo;
use linestring::{linestring_2d::Aabb2, linestring_3d::Plane};
use vector_traits::{
    approx::{AbsDiffEq, UlpsEq},
    glam::Vec3A,
    num_traits::AsPrimitive,
    GenericVector2, GenericVector3, HasXY, HasXYZ,
};
#[cfg(test)]
mod tests;

#[allow(clippy::type_complexity)]
fn parse_input<T: GenericVector3 + HasMatrix4>(
    input_model: &Model<'_>,
    cmd_arg_max_voronoi_dimension: T::Scalar,
) -> Result<
    (
        Vec<BV::Point<i64>>,
        Vec<BV::Line<i64>>,
        Aabb2<T::Vector2>,
        T::Matrix4Type,
    ),
    HallrError,
>
where
    FFIVector3: ConvertTo<T>,
{
    let mut aabb = linestring::linestring_3d::Aabb3::<T>::default();
    for v in input_model.vertices.iter() {
        aabb.update_with_point(v.to())
    }

    let (plane, transform, vor_aabb)= centerline::get_transform_relaxed(
        aabb,
        cmd_arg_max_voronoi_dimension,
        T::Scalar::default_epsilon(),
        T::Scalar::default_max_ulps(),
    ).map_err(|_|{
        let aabb_d:T = aabb.get_high().unwrap() - aabb.get_low().unwrap();
        let aabb_c:T = (aabb.get_high().unwrap() + aabb.get_low().unwrap())/2.0.into();
        HallrError::InputNotPLane(format!(
            "Input data not in one plane and/or plane not intersecting origin: Δ({},{},{}) C({},{},{})",
            aabb_d.x(), aabb_d.y(), aabb_d.z(), aabb_c.x(), aabb_c.y(), aabb_c.z()))
    })?;

    if plane != Plane::XY {
        return Err(HallrError::InvalidInputData(format!("At the moment the cmd_voronoi_diagram mesh operation only supports input data in the XY plane. {:?}", plane)));
    }

    let inverse_transform = transform.safe_inverse().ok_or(HallrError::InternalError(
        "Could not calculate inverse matrix".to_string(),
    ))?;

    println!(
        "cmd_voronoi_diagram: data was in plane:{:?} aabb:{:?}",
        plane, aabb
    );

    //println!("input Lines:{:?}", input_model.vertices);

    let mut vor_lines = Vec::<BV::Line<i64>>::with_capacity(input_model.indices.len() / 2);
    let vor_vertices: Vec<BV::Point<i64>> = input_model
        .vertices
        .iter()
        .map(|vertex| {
            let p = transform
                .transform_point3(T::new_3d(vertex.x.into(), vertex.y.into(), vertex.z.into()))
                .to_2d();
            BV::Point {
                x: p.x().as_(),
                y: p.y().as_(),
            }
        })
        .collect();
    let mut used_vertices = vob::Vob::<u32>::fill_with_false(vor_vertices.len());

    for chunk in input_model.indices.chunks(2) {
        let v0 = chunk[0];
        let v1 = chunk[1];

        vor_lines.push(BV::Line {
            start: vor_vertices[v0],
            end: vor_vertices[v1],
        });
        let _ = used_vertices.set(v0, true);
        let _ = used_vertices.set(v1, true);
    }
    // save the unused vertices as points
    let vor_vertices: Vec<BV::Point<i64>> = vor_vertices
        .into_iter()
        .enumerate()
        .filter(|x| !used_vertices[x.0])
        .map(|x| x.1)
        .collect();
    Ok((vor_vertices, vor_lines, vor_aabb, inverse_transform))
}

/// Runs boost cmd_voronoi_diagram over the input and generates to output model.
/// Removes the external edges as we can't handle infinite length edges in blender.
pub(crate) fn compute_voronoi_diagram(
    input_model: &Model<'_>,
    cmd_arg_max_voronoi_dimension: f32,
    cmd_discretization_distance: f32,
    cmd_arg_keep_input: bool,
) -> Result<(Vec<Vec3A>, Vec<usize>), HallrError> {
    let (vor_vertices, vor_lines, vor_aabb2, inverted_transform) =
        parse_input::<Vec3A>(input_model, cmd_arg_max_voronoi_dimension)?;
    let vor_diagram = {
        BV::Builder::<i64, f32>::default()
            .with_vertices(vor_vertices.iter())?
            .with_segments(vor_lines.iter())?
            .build()?
    };

    let discretization_distance: f32 = {
        let max_dist: <Vec3A as GenericVector3>::Vector2 =
            vor_aabb2.high().unwrap() - vor_aabb2.low().unwrap();
        cmd_discretization_distance * max_dist.magnitude() / 100.0
    };

    let reject_edges = voronoi_utils::reject_external_edges::<Vec3A>(&vor_diagram)?;
    let internal_vertices =
        voronoi_utils::find_internal_vertices::<Vec3A>(&vor_diagram, &reject_edges)?;
    let diagram_helper = voronoi_utils::DiagramHelperRo::<Vec3A> {
        vertices: vor_vertices,
        segments: vor_lines,
        diagram: vor_diagram,
        rejected_edges: reject_edges,
        internal_vertices,
        inverted_transform,
    };

    let (dhrw, mod_edges) = diagram_helper.convert_edges(discretization_distance)?;
    let (indices, vertices) =
        diagram_helper.generate_voronoi_edges_from_cells(dhrw, mod_edges, cmd_arg_keep_input)?;
    Ok((vertices, indices))
}

/// Run the voronoi_mesh command
pub(crate) fn process_command(
    config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    type Scalar = f32;

    if models.is_empty() {
        return Err(HallrError::InvalidInputData(
            "This operation requires ome input model".to_string(),
        ));
    }

    if models.len() > 1 {
        return Err(HallrError::InvalidInputData(
            "This operation only supports one model as input".to_string(),
        ));
    }

    let cmd_arg_max_voronoi_dimension: Scalar = config.get_mandatory_parsed_option(
        "MAX_VORONOI_DIMENSION",
        Some(super::DEFAULT_MAX_VORONOI_DIMENSION.as_()),
    )?;

    if !(super::DEFAULT_MAX_VORONOI_DIMENSION as i64..100_000_000)
        .contains(&cmd_arg_max_voronoi_dimension.as_())
    {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of MAX_VORONOI_DIMENSION is [{}..100_000_000[% :({})",
            super::DEFAULT_MAX_VORONOI_DIMENSION,
            cmd_arg_max_voronoi_dimension
        )));
    }
    let cmd_arg_discretization_distance: Scalar = config.get_mandatory_parsed_option(
        "DISTANCE",
        Some(super::DEFAULT_VORONOI_DISCRETE_DISTANCE.as_()),
    )?;

    if !(super::DEFAULT_VORONOI_DISCRETE_DISTANCE.as_()..5.0)
        .contains(&cmd_arg_discretization_distance)
    {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of DISTANCE is [{}..5.0[% :({})",
            super::DEFAULT_VORONOI_DISCRETE_DISTANCE,
            cmd_arg_discretization_distance
        )));
    }

    let cmd_arg_keep_input = config.get_parsed_option("KEEP_INPUT")?.unwrap_or(false);

    // used for simplification and discretization distance
    let max_distance: Scalar =
        cmd_arg_max_voronoi_dimension * cmd_arg_discretization_distance / 100.0;
    // we already tested a_command.models.len()
    let input_model = &models[0];
    if !input_model.has_identity_orientation() {
        return Err(HallrError::InvalidInputData(
            "The cmd_voronoi_diagram mesh operation currently requires identify world orientation"
                .to_string(),
        ));
    }

    // we already tested that there is only one model
    println!();
    println!("cmd_voronoi_mesh got command:");
    //println!("model.name:{:?}, ", input_model.name);
    println!("model.vertices:{:?}", input_model.vertices.len());
    println!("model.indices:{:?}", input_model.indices.len());
    println!(
        "model.world_orientation:{:?}:{}",
        input_model.world_orientation,
        input_model.has_identity_orientation()
    );
    println!("MAX_VORONOI_DIMENSION:{:?}", cmd_arg_max_voronoi_dimension);
    println!(
        "VORONOI_DISCRETE_DISTANCE:{:?}%",
        cmd_arg_discretization_distance
    );
    println!("KEEP_INPUT:{:?}", cmd_arg_keep_input);
    println!("max_distance:{:?}", max_distance);

    println!();

    // do the actual operation
    let (vertices, indices) = compute_voronoi_diagram(
        input_model,
        cmd_arg_max_voronoi_dimension,
        cmd_arg_discretization_distance,
        cmd_arg_keep_input,
    )?;
    let output_model = OwnedModel {
        world_orientation: Model::copy_world_orientation(input_model)?,
        indices,
        vertices: vertices
            .into_iter()
            .map(|mut v: Vec3A| {
                v.set_z(0.0);
                v.to()
            })
            .collect(),
    };

    let mut return_config = ConfigType::new();
    let _ = return_config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = return_config.insert("REMOVE_DOUBLES".to_string(), "true".to_string());

    println!(
        "cmd_voronoi_diagram mesh operation returning {} vertices, {} indices",
        output_model.vertices.len(),
        output_model.indices.len()
    );
    Ok((
        output_model.vertices,
        output_model.indices,
        output_model.world_orientation.to_vec(),
        return_config,
    ))
}
