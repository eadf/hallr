// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{ConfigType, Model, Options, OwnedModel};
use crate::{HallrError, ffi, ffi::FFIVector3, utils};
use boostvoronoi as BV;
use boostvoronoi::OutputType;
use hronn::prelude::*;
use itertools::Itertools;
use linestring::{linestring_3d::LineString3, prelude::LineString2};
use rayon::{
    iter::ParallelIterator,
    prelude::{IntoParallelIterator, IntoParallelRefIterator},
};
use vector_traits::{
    approx::{AbsDiffEq, UlpsEq},
    num_traits::{AsPrimitive, NumCast, real::Real},
    prelude::{
        Aabb3, Affine3D, GenericScalar, GenericVector2, GenericVector3, HasXY, HasXYZ, Plane,
    },
};

#[cfg(test)]
mod tests;

#[inline(always)]
/// make a key from v0 and v1, lowest index will always be first
fn make_edge_key(v0: usize, v1: usize) -> (usize, usize) {
    if v0 < v1 { (v0, v1) } else { (v1, v0) }
}

/// reformat the input into a useful structure
#[allow(clippy::type_complexity)]
fn parse_input<T: GenericVector3>(
    model: &Model<'_>,
) -> Result<
    (
        ahash::AHashSet<(usize, usize)>,
        Vec<T>,
        <T as GenericVector3>::Aabb,
    ),
    HallrError,
>
where
    FFIVector3: ConvertTo<T>,
{
    let mut aabb = <T as GenericVector3>::Aabb::default();
    for v in model.vertices.iter() {
        aabb.add_point(v.to())
    }

    let plane =
        aabb.get_plane_relaxed(T::Scalar::default_epsilon(), T::Scalar::default_max_ulps()).ok_or_else(|| {
            let aabbe_d = aabb.max() - aabb.min();
            let aabbe_c = aabb.center();
            HallrError::InputNotPLane(format!(
                "Input data not in one plane and/or plane not intersecting origin: Δ({},{},{}) C({},{},{})",
                aabbe_d.x(), aabbe_d.y(), aabbe_d.z(),aabbe_c.x(), aabbe_c.y(), aabbe_c.z()
            ))
        })?;
    println!("Centerline op: data was in plane:{plane:?} aabb:{aabb:?}",);
    //println!("vertices:{:?}", model.vertices);
    //println!("indices:{:?}", model.indices);
    let mut edge_set = ahash::AHashSet::<(usize, usize)>::default();

    for edge in model.indices.chunks(2) {
        let v0 = edge[0];
        let v1 = edge[1];
        let key = make_edge_key(v0, v1);
        let _ = edge_set.insert(key);
    }
    let mut converted_vertices = Vec::<T>::with_capacity(model.vertices.len());
    for p in model.vertices.iter() {
        if !p.is_finite() {
            return Err(HallrError::InvalidInputData(format!(
                "Only valid coordinates are allowed ({},{},{})",
                p.x(),
                p.y(),
                p.z()
            )));
        } else {
            converted_vertices.push(p.to())
        }
    }

    Ok((edge_set, converted_vertices, aabb))
}

/// Build the return model
#[allow(clippy::type_complexity)]
fn build_output_model<T>(
    _a_command: &ConfigType,
    shapes: Vec<(
        centerline::LineStringSet2<T::Vector2>,
        centerline::Centerline<i64, T>,
    )>,
    cmd_arg_weld: bool,
    inverted_transform: T::Affine,
    cmd_arg_negative_radius: bool,
    cmd_arg_keep_input: bool,
    world_to_local: Option<impl Fn(FFIVector3) -> FFIVector3>,
) -> Result<OwnedModel, HallrError>
where
    T: GenericVector3 + ConvertTo<FFIVector3>,
    T::Scalar: OutputType,
{
    //let input_pb_model = &a_command.models[0];

    let estimated_capacity: usize = (shapes
        .iter()
        .map::<usize, _>(|(ls, cent)| {
            ls.set().iter().map(|ls| ls.len()).sum::<usize>()
                + cent.lines.iter().flatten().count()
                + cent
                    .line_strings
                    .iter()
                    .flatten()
                    .map(|ls| ls.len())
                    .sum::<usize>()
        })
        .sum::<usize>()
        * 5)
        / 4;

    let mut output_model_edges = Vec::<(u32, u32)>::with_capacity(estimated_capacity);

    // map between vertex and vertex index
    let mut v_map = utils::VertexDeduplicator3D::<T>::default();

    for shape in shapes {
        // Draw the input segments
        if cmd_arg_keep_input {
            for input_linestring in shape.0.set().iter() {
                if input_linestring.len() < 3 {
                    return Err(HallrError::InternalError(
                        "Linestring with less than 3 points found (loop-around vertex is repeated)"
                            .to_string(),
                    ));
                }
                //println!("Input linestring: {:?}", input_linestring.0);
                //let input_linestring = &input_linestring.0;
                //println!("Input linestring: {:?}", input_linestring);
                //println!("output_model_vertices:{:?}",output_model_vertices);

                for (v0, v1) in input_linestring.iter().tuple_windows::<(_, _)>() {
                    let v0 = v0.to_3d(T::Scalar::ZERO);
                    let i0 = v_map.get_index_or_insert(v0)?;
                    let v1 = v1.to_3d(T::Scalar::ZERO);
                    let i1 = v_map.get_index_or_insert(v1)?;

                    //println!("input edge: {}-{}", p.0, p.1);
                    output_model_edges.push((i0, i1));
                }
            }
        }

        if !cmd_arg_weld {
            // Do not share any vertices between input geometry and center line if cmd_arg_weld is false
            v_map.clear_dedup_cache()
        }

        // draw the straight edges of the voronoi output
        for line in shape.1.lines.iter().flatten() {
            let v0 = line.start;
            let v1 = line.end;
            if v0 == v1 {
                continue;
            }
            let v0_index = v_map.get_index_or_insert(v0)?;
            let v1_index = v_map.get_index_or_insert(v1)?;

            if v0_index == v1_index {
                println!(
                    "v0_index==v1_index, but v0!=v1 v0:{v0:?} v1:{v1:?} v0_index:{v0_index:?} v1_index:{v1_index:?}",
                );
                continue;
            }
            output_model_edges.push((v0_index, v1_index));
        }

        // draw the concatenated line strings of the voronoi output
        for linestring in shape.1.line_strings.iter().flatten() {
            if linestring.len() < 2 {
                return Err(HallrError::InternalError(
                    "Linestring with less than 2 points found".to_string(),
                ));
            }
            // unwrap of first and last is safe now that we know there are at least 2 vertices in the list
            let v0 = linestring.first().unwrap();
            let v1 = linestring.last().unwrap();
            let v0_index = v_map.get_index_or_insert(*v0)?;
            let v1_index = v_map.get_index_or_insert(*v1)?;
            // we only need to lookup the start and end points for vertex duplication
            let vertex_index_iterator = Some(v0_index)
                .into_iter()
                .chain(
                    linestring
                        .iter()
                        .skip(1)
                        .take(linestring.len() - 2)
                        .map(|p| v_map.insert_and_get_index(*p)),
                )
                .chain(Some(v1_index).into_iter());
            for p in vertex_index_iterator.tuple_windows::<(_, _)>() {
                output_model_edges.push((p.0, p.1));
            }
        }
    }
    //println!("allocated {} needed {} and {}", count, output_pb_model_vertices.len(), output_pb_model_faces.len());
    // Todo: store in the output_indices format in the first place
    let mut output_pb_model_indices = Vec::<usize>::with_capacity(output_model_edges.len() * 2);
    for (a, b) in output_model_edges {
        if a != b {
            output_pb_model_indices.push(a as usize);
            output_pb_model_indices.push(b as usize);
        } else {
            println!("Something is wrong wanted to add edge {a} to {b}");
        }
    }
    //println!("Resulting centerline model:{:?}", output_pb_model_indices);
    /*for p in output_pb_model_indices.chunks(2) {
        print!("{}-{}, ", p[0], p[1]);
    }
    println!();*/
    let output_model_vertices: Vec<FFIVector3> = if let Some(world_to_local) = world_to_local {
        if cmd_arg_negative_radius {
            v_map
                .vertices
                .into_iter()
                .map(|v| world_to_local(inverted_transform.transform_point3(v).to()))
                .collect()
        } else {
            v_map
                .vertices
                .into_iter()
                .map(|v| {
                    let point = inverted_transform.transform_point3(v);
                    world_to_local(T::new_3d(point.x(), point.y(), -point.z()).to())
                })
                .collect()
        }
    } else if cmd_arg_negative_radius {
        v_map
            .vertices
            .into_iter()
            .map(|v| inverted_transform.transform_point3(v).to())
            .collect()
    } else {
        v_map
            .vertices
            .into_iter()
            .map(|v| {
                let point = inverted_transform.transform_point3(v);
                T::new_3d(point.x(), point.y(), -point.z()).to()
            })
            .collect()
    };

    Ok(OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        //name: input_pb_model.name.clone(),
        vertices: output_model_vertices,
        indices: output_pb_model_indices,
    })
}

/// Run the centerline command
pub(crate) fn process_command<T>(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError>
where
    T: GenericVector3,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    T::Scalar: OutputType,
    i64: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<i64>,
{
    let default_max_voronoi_dimension: T::Scalar =
        NumCast::from(super::DEFAULT_MAX_VORONOI_DIMENSION).unwrap();

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::LineChunks)?;

    // angle is supposed to be in degrees
    let cmd_arg_angle: T::Scalar = input_config.get_mandatory_parsed_option("ANGLE", None)?;
    if !(0.0.into()..=90.0.into()).contains(&cmd_arg_angle) {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of ANGLE is [0..90] :({cmd_arg_angle})",
        )));
    }
    let cmd_arg_remove_internals = input_config
        .get_parsed_option::<bool>("REMOVE_INTERNALS")?
        .unwrap_or(true);

    let cmd_arg_discrete_distance = input_config.get_mandatory_parsed_option("DISTANCE", None)?;
    if !(0.001.into()..100.0.into()).contains(&cmd_arg_discrete_distance) {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of DISTANCE is [0.001..100[% :({cmd_arg_discrete_distance:?})",
        )));
    }
    let cmd_arg_max_voronoi_dimension = input_config
        .get_parsed_option::<T::Scalar>("MAX_VORONOI_DIMENSION")?
        .unwrap_or(default_max_voronoi_dimension);
    if !(default_max_voronoi_dimension..100_000_000.0.into())
        .contains(&cmd_arg_max_voronoi_dimension)
    {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of MAX_VORONOI_DIMENSION is [{}..100_000_000[% :({})",
            super::DEFAULT_MAX_VORONOI_DIMENSION,
            cmd_arg_max_voronoi_dimension
        )));
    }
    let cmd_arg_simplify = input_config
        .get_parsed_option::<bool>("SIMPLIFY")?
        .unwrap_or(true);

    let (cmd_arg_weld, cmd_arg_keep_input) = {
        let mut cmd_arg_weld = input_config.get_parsed_option("WELD")?.unwrap_or(true);
        let cmd_arg_keep_input = input_config
            .get_parsed_option("KEEP_INPUT")?
            .unwrap_or(true);

        if !cmd_arg_keep_input {
            // cmd_arg_keep_input overrides cmd_arg_weld
            cmd_arg_weld = false;
        }
        (cmd_arg_weld, cmd_arg_keep_input)
    };

    let cmd_arg_negative_radius = input_config
        .get_parsed_option::<bool>("NEGATIVE_RADIUS")?
        .unwrap_or(true);

    // used for simplification and discretization distance
    let max_distance = cmd_arg_max_voronoi_dimension * cmd_arg_discrete_distance / 100.0.into();

    if models.is_empty() {
        return Err(HallrError::InvalidInputData(
            "No models detected".to_string(),
        ));
    }
    let input_model = models.first().unwrap();
    if input_model.indices.is_empty() || input_model.vertices.is_empty() {
        return Err(HallrError::InvalidInputData(
            "Model did not contain any data".to_string(),
        ));
    }

    /*if !model.has_identity_orientation() {
        return Err(HallrError::InvalidInputData(
            "The centerline operation currently requires identify world orientation".to_string(),
        ));
    }*/
    // The dot product between normalized vectors of edge and the segment that created it.
    // Can also be described as cos(angle) between edge and segment.
    let dot_limit = cmd_arg_angle.to_radians().cos().abs();

    println!("Rust: cmd_centerline got command");
    println!("Rust: model.vertices:{:?}", input_model.vertices.len());
    println!("Rust: model.indices:{:?}", input_model.indices.len());
    println!(
        "Rust: model.world_orientation:{:?}:{}",
        input_model.world_orientation,
        input_model.has_identity_orientation()
    );
    println!("Rust: ANGLE:{cmd_arg_angle:?}°, dot_limit:{dot_limit:?}");
    println!("Rust: REMOVE_INTERNALS:{cmd_arg_remove_internals:?}");
    println!("Rust: SIMPLIFY:{cmd_arg_simplify:?}");
    println!("Rust: KEEP_INPUT:{cmd_arg_keep_input:?}, WELD:{cmd_arg_weld:?}",);
    println!("Rust: DISTANCE:{cmd_arg_discrete_distance:?}%");
    println!("Rust: NEGATIVE_RADIUS:{cmd_arg_negative_radius:?}");
    println!("Rust: MAX_VORONOI_DIMENSION:{cmd_arg_max_voronoi_dimension:?}");
    println!("Rust: max_distance:{max_distance:?}");
    println!();

    //let mut obj = Obj::<FFIVector3>::new("cmd_centerline");
    //println!("rust: vertices.len():{}", vertices.len());
    //println!("rust: indices.len():{}", indices.len());
    //println!("rust: indices:{:?}", model.indices);

    // convert the input vertices to 2d point cloud
    //let vertices: Vec<T::Vector2> = vertices.iter().map(|v| v.to().to_2d()).collect();
    //println!("Vertices:{:?}", vertices);
    //println!("Indices:{:?}", indices);

    let (edges, vertices, total_aabb) = parse_input(input_model)?;
    //println!("edge set: {:?}", edges);
    //println!("-> divide_into_shapes");
    let lines = centerline::divide_into_shapes(edges, vertices)?;
    //println!("-> get_transform_relaxed");
    let (_plane, transform, _voronoi_input_aabb) = centerline::get_transform_relaxed::<T>(
        total_aabb,
        cmd_arg_max_voronoi_dimension,
        T::Scalar::default_epsilon(),
        T::Scalar::default_max_ulps(),
    )?;

    let inverted_transform = transform.try_inverse().ok_or(HallrError::InternalError(
        "Could not generate the inverse matrix.".to_string(),
    ))?;

    //println!("-> transform");
    /*for s in lines.iter() {
        println!("3d line: {:?}", s.set);
    }*/

    // transform each linestring to 2d
    let mut lines_as_2d: Vec<centerline::LineStringSet2<T::Vector2>> = lines
        .par_iter()
        .map(|x| {
            let mut xc = x.clone();
            xc.apply(&|v| transform.transform_point3(v));
            xc.copy_to_2d(Plane::XY)
        })
        .collect();
    {
        // round the floats to nearest int
        let round_float = |v: <T as GenericVector3>::Vector2| -> <T as GenericVector3>::Vector2 {
            <T as GenericVector3>::Vector2::new_2d(v.x().round(), v.y().round())
        };
        for r in lines_as_2d.iter_mut() {
            r.apply(&round_float);
        }
    }
    //for s in lines_as_2d.iter() {
    //    println!("2d line: {:?}", s.set());
    //}

    // calculate the hull of each shape
    let lines_as_2d: Vec<centerline::LineStringSet2<T::Vector2>> = lines_as_2d
        .into_par_iter()
        .map(|mut x| {
            let _ = x.calculate_convex_hull();
            x
        })
        .collect();

    //println!("Started with {} shapes", raw_data.len());
    let lines_as_2d = centerline::consolidate_shapes(lines_as_2d)?;

    let shapes = lines_as_2d
        .into_par_iter()
        .map(|shape| {
            let mut segments =
                Vec::<BV::Line<i64>>::with_capacity(shape.set().iter().map(|x| x.len()).sum());
            for lines in shape.set().iter() {
                for lineseq in lines.window_iter() {
                    segments.push(BV::Line::new(
                        // boost voronoi only accepts integers as coordinates
                        BV::Point {
                            x: lineseq.start.x().as_(),
                            y: lineseq.start.y().as_(),
                        },
                        BV::Point {
                            x: lineseq.end.x().as_(),
                            y: lineseq.end.y().as_(),
                        },
                    ))
                }
            }
            let mut c = centerline::Centerline::<i64, T>::with_segments(segments);
            if let Err(centerline_error) = c.build_voronoi() {
                return Err(centerline_error.into());
            }
            if cmd_arg_remove_internals {
                if let Err(centerline_error) =
                    c.calculate_centerline(dot_limit, max_distance, shape.get_internals())
                {
                    return Err(centerline_error.into());
                }
            } else if let Err(centerline_error) =
                c.calculate_centerline(dot_limit, max_distance, None)
            {
                return Err(centerline_error.into());
            }

            if cmd_arg_simplify && c.line_strings.is_some() {
                // simplify every line string with rayon
                c.line_strings = Some(
                    c.line_strings
                        .take()
                        .unwrap()
                        .into_par_iter()
                        .map(|ls| {
                            //let pre = ls.len();
                            ls.simplify_rdp(max_distance)
                            ////println!("simplified ls from {} to {}", pre, ls.len());
                            //ls
                        })
                        .collect(),
                );
            }
            Ok((shape, c))
        })
        .collect::<Result<
            Vec<(
                centerline::LineStringSet2<T::Vector2>,
                centerline::Centerline<i64, T>,
            )>,
            HallrError,
        >>()?;

    let model = build_output_model(
        &input_config,
        shapes,
        cmd_arg_weld,
        inverted_transform,
        cmd_arg_negative_radius,
        cmd_arg_keep_input,
        input_model.get_world_to_local_transform()?,
    )?;

    //println!("result vertices:{:?}", obj.vertices);
    //println!("result edges:{:?}", obj.lines.first());
    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::LineChunks.to_string(),
    );
    if let Some(mv) = input_config.get_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }
    println!(
        "centerline operation returning {} vertices, {} indices",
        model.vertices.len(),
        model.indices.len()
    );
    //println!("rv:vertices:{:?}", model.vertices);
    //println!("rv:indices:{:?}", model.indices);
    Ok((
        model.vertices,
        model.indices,
        model.world_orientation.to_vec(),
        return_config,
    ))
}
