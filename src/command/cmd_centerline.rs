#[cfg(test)]
mod tests;

use super::{ConfigType, Model, Options, OwnedModel};
use crate::{geo::HashableVector2, prelude::*};
use boostvoronoi as BV;
use boostvoronoi::OutputType;
use centerline::{HasMatrix4, Matrix4};
use hronn::prelude::*;
use itertools::Itertools;
use linestring::linestring_3d::{Aabb3, Plane};
use rayon::prelude::*;
use vector_traits::{
    approx::{AbsDiffEq, UlpsEq},
    num_traits::{real::Real, AsPrimitive, NumCast},
    GenericScalar, GenericVector2, GenericVector3, HasXY, HasXYZ,
};

/// converts to a private, comparable and hashable format
/// only use this for floats that are f32::is_finite()
/// This will only work for floats that's identical in every bit.
/// The z coordinate will not be used because it might be slightly different
/// depending on how it was calculated. Not using z will also make the calculations faster.
#[inline(always)]
fn transmute_to_u32<T: HasXYZ>(a: &T) -> (u32, u32) {
    let x: f32 = a.x().as_();
    let y: f32 = a.y().as_();
    (x.to_bits(), y.to_bits())
}

#[inline(always)]
/// make a key from v0 and v1, lowest index will always be first
fn make_edge_key(v0: usize, v1: usize) -> (usize, usize) {
    if v0 < v1 {
        (v0, v1)
    } else {
        (v1, v0)
    }
}

/// reformat the input into a useful structure
#[allow(clippy::type_complexity)]
fn parse_input<T: GenericVector3>(
    model: &Model<'_>,
) -> Result<(ahash::AHashSet<(usize, usize)>, Vec<T>, Aabb3<T>), HallrError>
where
    FFIVector3: ConvertTo<T>,
{
    let mut aabb = Aabb3::<T>::default();
    for v in model.vertices.iter() {
        aabb.update_point(v.to())
    }

    let plane =
        Plane::get_plane_relaxed(aabb, T::Scalar::default_epsilon(), T::Scalar::default_max_ulps()).ok_or_else(|| {
            let aabbe_d = aabb.get_high().unwrap() - aabb.get_low().unwrap();
            let aabbe_c = (aabb.get_high().unwrap() + aabb.get_low().unwrap())/T::Scalar::TWO;
            HallrError::InputNotPLane(format!(
                "Input data not in one plane and/or plane not intersecting origin: Δ({},{},{}) C({},{},{})",
                aabbe_d.x(), aabbe_d.y(), aabbe_d.z(),aabbe_c.x(), aabbe_c.y(), aabbe_c.z()
            ))
        })?;
    println!(
        "Centerline op: data was in plane:{:?} aabb:{:?}",
        plane, aabb
    );
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
        if !p.x().is_finite() || !p.y().is_finite() || !p.z().is_finite() {
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
fn build_output_model<T: GenericVector3>(
    _a_command: &ConfigType,
    shapes: Vec<(
        linestring::linestring_2d::LineStringSet2<T::Vector2>,
        centerline::Centerline<i64, T>,
    )>,
    cmd_arg_weld: bool,
    inverted_transform: T::Matrix4Type,
    cmd_arg_negative_radius: bool,
    cmd_arg_keep_input: bool,
) -> Result<OwnedModel, HallrError>
where
    T: HasMatrix4 + ConvertTo<FFIVector3>,
    T::Scalar: OutputType,
{
    let transform_point = |point: T| -> T {
        if cmd_arg_negative_radius {
            inverted_transform.transform_point3(point)
        } else {
            let point = inverted_transform.transform_point3(point);
            T::new_3d(point.x(), point.y(), -point.z())
        }
    };

    //let input_pb_model = &a_command.models[0];

    let estimated_capacity: usize = (shapes
        .iter()
        .map::<usize, _>(|(ls, cent)| {
            ls.set().iter().map(|ls| ls.points().len()).sum::<usize>()
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

    let mut output_model_vertices = Vec::<FFIVector3>::with_capacity(estimated_capacity);
    let mut output_model_edges = Vec::<(u32, u32)>::with_capacity(estimated_capacity);

    // map between 'meta-vertex' and vertex index
    let mut v_map = ahash::AHashMap::<(u32, u32), usize>::default();

    for shape in shapes {
        // Draw the input segments
        if cmd_arg_keep_input {
            for input_linestring in shape.0.set().iter() {
                if input_linestring.points().len() < 3 {
                    return Err(HallrError::InternalError(
                        "Linestring with less than 3 points found (loop-around vertex is repeated)"
                            .to_string(),
                    ));
                }
                //println!("Input linestring: {:?}", input_linestring.0);
                //let input_linestring = &input_linestring.0;
                //println!("Input linestring: {:?}", input_linestring);
                //println!("output_model_vertices:{:?}",output_model_vertices);

                let vertex_index_iterator = input_linestring.0.iter().map(|p| {
                    let v2 = p.to_3d(T::Scalar::ZERO);
                    let v2_key = transmute_to_u32(&v2);
                    //println!("testing {:?} as key {:?}", v2, v2_key);
                    let v2_index = *v_map.entry(v2_key).or_insert_with(|| {
                        let new_index = output_model_vertices.len();
                        output_model_vertices.push(inverted_transform.transform_point3(v2).to());
                        //println!("i2 pushed ({},{},{}) as {}", v2.x(), v2.y(), v2.z(), new_index);
                        new_index
                    });
                    v2_index
                });
                for p in vertex_index_iterator.tuple_windows::<(_, _)>() {
                    //println!("input edge: {}-{}", p.0, p.1);
                    output_model_edges.push((p.0 as u32, p.1 as u32));
                }
            }
        }

        if !cmd_arg_weld {
            // Do not share any vertices between input geometry and center line if cmd_arg_weld is false
            v_map.clear()
        }

        // draw the straight edges of the voronoi output
        for line in shape.1.lines.iter().flatten() {
            let v0 = line.start;
            let v1 = line.end;
            if v0 == v1 {
                continue;
            }
            let v0_key = transmute_to_u32(&v0);
            let v0_index = *v_map.entry(v0_key).or_insert_with(|| {
                let new_index = output_model_vertices.len();
                output_model_vertices.push(transform_point(v0).to());
                //println!("s0 pushed ({},{},{})", v0.x, v0.y, v0.z);
                new_index
            });

            let v1_key = transmute_to_u32(&v1);
            let v1_index = *v_map.entry(v1_key).or_insert_with(|| {
                let new_index = output_model_vertices.len();

                output_model_vertices.push(transform_point(v1).to());
                //println!("s1 pushed ({},{},{})", v1.x, v1.y, v1.z);
                new_index
            });

            if v0_index == v1_index {
                println!(
                    "v0_index==v1_index, but v0!=v1 v0:{:?} v1:{:?} v0_index:{:?} v1_index:{:?}",
                    v0, v1, v0_index, v1_index
                );
                continue;
            }
            output_model_edges.push((v0_index as u32, v1_index as u32));
        }

        // draw the concatenated line strings of the voronoi output
        for linestring in shape.1.line_strings.iter().flatten() {
            if linestring.points().len() < 2 {
                return Err(HallrError::InternalError(
                    "Linestring with less than 2 points found".to_string(),
                ));
            }
            // unwrap of first and last is safe now that we know there are at least 2 vertices in the list
            let v0 = linestring.points().first().unwrap();
            let v1 = linestring.points().last().unwrap();
            let v0_key = transmute_to_u32(v0);
            let v0_index = *v_map.entry(v0_key).or_insert_with(|| {
                let new_index = output_model_vertices.len();
                output_model_vertices.push(transform_point(*v0).to());
                //println!("ls0 pushed ({},{},{})", v0.x, v0.y, v0.z);
                new_index
            });
            let v1_key = transmute_to_u32(v1);
            let v1_index = *v_map.entry(v1_key).or_insert_with(|| {
                let new_index = output_model_vertices.len();

                output_model_vertices.push(transform_point(*v1).to());
                new_index
            });
            // we only need to lookup the start and end points for vertex duplication
            let vertex_index_iterator = Some(v0_index)
                .into_iter()
                .chain(
                    linestring
                        .points()
                        .iter()
                        .skip(1)
                        .take(linestring.points().len() - 2)
                        .map(|p| {
                            let new_index = output_model_vertices.len();
                            output_model_vertices.push(transform_point(*p).to());
                            new_index
                        }),
                )
                .chain(Some(v1_index).into_iter());
            for p in vertex_index_iterator.tuple_windows::<(_, _)>() {
                output_model_edges.push((p.0 as u32, p.1 as u32));
            }
        }
    }
    //println!("allocated {} needed {} and {}", count, output_pb_model_vertices.len(), output_pb_model_faces.len());
    // Todo: store in the output_pb_model_indices format in the first place
    let mut output_pb_model_indices = Vec::<usize>::with_capacity(output_model_edges.len() * 2);
    for (a, b) in output_model_edges {
        if a != b {
            output_pb_model_indices.push(a as usize);
            output_pb_model_indices.push(b as usize);
        } else {
            println!("Something is wrong wanted to add edge {} to {}", a, b);
        }
    }
    //println!("Resulting centerline model:{:?}", output_pb_model_indices);
    /*for p in output_pb_model_indices.chunks(2) {
        print!("{}-{}, ", p[0], p[1]);
    }
    println!();*/
    Ok(OwnedModel {
        //name: input_pb_model.name.clone(),
        //world_orientation: input_pb_model.world_orientation.clone(),
        vertices: output_model_vertices,
        indices: output_pb_model_indices,
    })
}

/// Run the centerline command
pub(crate) fn process_command<T: GenericVector3>(
    config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError>
where
    T: ConvertTo<FFIVector3> + HasMatrix4,
    FFIVector3: ConvertTo<T>,
    HashableVector2: From<T::Vector2>,
    T::Scalar: OutputType,
    i64: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<i64>,
{
    let default_max_voronoi_dimension: T::Scalar =
        NumCast::from(super::DEFAULT_MAX_VORONOI_DIMENSION).unwrap();
    if false {
        // use this to generate test data from blender
        println!("Some test data:");
        println!("model:indices:{:?}", models[0].indices);
        println!("model:vertices:{:?}", models[0].vertices);
        println!("config{:?}", config);
    }
    // angle is supposed to be in degrees
    let cmd_arg_angle: T::Scalar = config.get_mandatory_parsed_option("ANGLE")?;
    if !(0.0.into()..=90.0.into()).contains(&cmd_arg_angle) {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of ANGLE is [0..90] :({})",
            cmd_arg_angle
        )));
    }
    let cmd_arg_remove_internals = config
        .get_parsed_option::<bool>("REMOVE_INTERNALS")?
        .unwrap_or(true);

    let cmd_arg_discrete_distance = config.get_mandatory_parsed_option("DISTANCE")?;
    if !(0.004.into()..100.0.into()).contains(&cmd_arg_discrete_distance) {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of DISTANCE is [0.005..100[% :({:?})",
            cmd_arg_discrete_distance
        )));
    }
    let cmd_arg_max_voronoi_dimension = config
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
    let cmd_arg_simplify = config
        .get_parsed_option::<bool>("SIMPLIFY")?
        .unwrap_or(true);

    let (cmd_arg_weld, cmd_arg_keep_input) = {
        let mut cmd_arg_weld = config.get_parsed_option("WELD")?.unwrap_or(true);
        let cmd_arg_keep_input = config.get_parsed_option("KEEP_INPUT")?.unwrap_or(true);

        if !cmd_arg_keep_input {
            // cmd_arg_keep_input overrides cmd_arg_weld
            cmd_arg_weld = false;
        }
        (cmd_arg_weld, cmd_arg_keep_input)
    };

    let cmd_arg_negative_radius = config
        .get_parsed_option::<bool>("NEGATIVE_RADIUS")?
        .unwrap_or(true);

    let mesh_format = config.get_mandatory_option("mesh.format")?;
    if mesh_format.ne("line_chunks") {
        return Err(HallrError::InvalidInputData(
            "Model mesh data must be in the 'line_chunks' format".to_string(),
        ));
    }

    // used for simplification and discretization distance
    let max_distance = cmd_arg_max_voronoi_dimension * cmd_arg_discrete_distance / 100.0.into();

    if models.is_empty() {
        return Err(HallrError::InvalidInputData(
            "No models detected".to_string(),
        ));
    }
    let model = models.first().unwrap();
    if model.indices.is_empty() || model.vertices.is_empty() {
        return Err(HallrError::InvalidInputData(
            "Model did not contain any data".to_string(),
        ));
    }

    // The dot product between normalized vectors of edge and the segment that created it.
    // Can also be described as cos(angle) between edge and segment.
    let dot_limit = cmd_arg_angle.to_radians().cos().abs();

    println!("cmd_centerline got command");
    //println!("model.vertices:{:?}", model.vertices.len());
    //println!("model.indices:{:?}", model.indices.len());
    //println!("model.faces:{:?}", faces.len());
    /*println!(
        "model.world_orientation:{:?}",
        model.world_orientation.as_ref().map_or(0, |_| 16)
    );*/
    println!("ANGLE:{:?}°", cmd_arg_angle);
    println!("dot_limit:{:?}", dot_limit);
    println!("REMOVE_INTERNALS:{:?}", cmd_arg_remove_internals);
    println!("SIMPLIFY:{:?}", cmd_arg_simplify);
    println!("WELD:{:?}", cmd_arg_weld);
    println!("KEEP_INPUT:{:?}", cmd_arg_keep_input);
    println!("DISTANCE:{:?}%", cmd_arg_discrete_distance);
    println!("NEGATIVE_RADIUS:{:?}", cmd_arg_negative_radius);
    println!("MAX_VORONOI_DIMENSION:{:?}", cmd_arg_max_voronoi_dimension);
    println!("max_distance:{:?}", max_distance);
    println!();

    //let mut obj = Obj::<FFIVector3>::new("cmd_centerline");
    //println!("rust: vertices.len():{}", vertices.len());
    //println!("rust: indices.len():{}", indices.len());
    //println!("rust: indices:{:?}", model.indices);

    // convert the input vertices to 2d point cloud
    //let vertices: Vec<T::Vector2> = vertices.iter().map(|v| v.to().to_2d()).collect();
    //println!("Vertices:{:?}", vertices);
    //println!("Indices:{:?}", indices);

    let (edges, vertices, total_aabb) = parse_input(model)?;
    //println!("edge set: {:?}", edges);
    //println!("-> divide_into_shapes");
    let lines = centerline::divide_into_shapes(edges, vertices)?;
    //println!("-> get_transform_relaxed");
    let (_plane, transform, _voronoi_input_aabb) = centerline::get_transform_relaxed(
        total_aabb,
        cmd_arg_max_voronoi_dimension,
        T::Scalar::default_epsilon(),
        T::Scalar::default_max_ulps(),
    )?;

    let inverted_transform = transform.safe_inverse().ok_or(HallrError::InternalError(
        "Could not generate the inverse matrix.".to_string(),
    ))?;

    //println!("-> transform");
    /*for s in lines.iter() {
        println!("3d line: {:?}", s.set);
    }*/

    // transform each linestring to 2d
    let mut lines_as_2d: Vec<linestring::linestring_2d::LineStringSet2<T::Vector2>> = lines
        .par_iter()
        .map(|x| {
            x.clone()
                .apply(&|v| transform.transform_point3(v))
                .copy_to_2d(Plane::XY)
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
    let lines_as_2d: Vec<linestring::linestring_2d::LineStringSet2<T::Vector2>> = lines_as_2d
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
                Vec::<BV::Line<i64>>::with_capacity(shape.set().iter().map(|x| x.0.len()).sum());
            for lines in shape.set().iter() {
                for lineseq in lines.iter() {
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
                linestring::linestring_2d::LineStringSet2<T::Vector2>,
                centerline::Centerline<i64, T>,
            )>,
            HallrError,
        >>()?;
    //println!("<-build_voronoi");
    let model = build_output_model(
        &config,
        shapes,
        cmd_arg_weld,
        inverted_transform,
        cmd_arg_negative_radius,
        cmd_arg_keep_input,
    )?;

    //println!("result vertices:{:?}", obj.vertices);
    //println!("result edges:{:?}", obj.lines.first());
    let mut return_config = ConfigType::new();
    let _ = return_config.insert("mesh.format".to_string(), "line_chunks".to_string());
    println!(
        "centerline operation returning {} vertices, {} indices",
        model.vertices.len(),
        model.indices.len()
    );
    //println!("rv:vertices:{:?}", model.vertices);
    //println!("rv:indices:{:?}", model.indices);
    Ok((model.vertices, model.indices, return_config))
}
