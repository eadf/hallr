use super::{ConfigType, Model, Options};
use crate::{prelude::*, utils::HashableVector2};
use ahash::{AHashMap, AHashSet};
use hronn::prelude::ConvertTo;
//use itertools::Itertools;
use linestring::linestring_3d::{LineString3, Plane};
use smallvec::SmallVec;
use std::collections::BTreeMap;
use linestring::prelude::LineString2;
//use linestring::prelude::LineString2;
use vector_traits::{
    num_traits::AsPrimitive, GenericScalar, GenericVector2, GenericVector3, HasXY, HasXYZ,
};

#[cfg(test)]
mod tests;

/// converts to a private, comparable and hashable format
/// only use this for floats that are f32::is_finite()
/// This will only work for floats that's identical in every bit.
/// The z coordinate will not be used because it might be slightly different
/// depending on how it was calculated. Not using z will also make the calculations faster.
// todo: replace with utils function
#[inline(always)]
fn transmute_xy_to_u32<T: HasXYZ>(a: &T) -> (u32, u32) {
    let x: f32 = a.x().as_();
    let y: f32 = a.y().as_();
    (x.to_bits(), y.to_bits())
}

/// converts to a private, comparable and hashable format
/// only use this for floats that are f32::is_finite()
/// This will only work for floats that's identical in every bit.
#[inline(always)]
fn transmute_xyz_to_u32<T: HasXYZ>(a: &T) -> (u32, u32, u32) {
    let x: f32 = a.x().as_();
    let y: f32 = a.y().as_();
    let z: f32 = a.z().as_();
    (x.to_bits(), y.to_bits(), z.to_bits())
}

/// reformat the input into converted vertices.
fn parse_input<T: GenericVector3>(model: &Model<'_>) -> Result<Vec<T>, HallrError>
where
    FFIVector3: ConvertTo<T>,
{
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

    Ok(converted_vertices)
}

/// Follows a detected line starting at `current`. If `next` is Some, that vertex id should be picked next.
/// `adjacency_map`: contains a map by vertex id key, and a list of adjacent vertices as `value`
/// `termination_nodes`: a set of nodes where lines should end. Such vertices has no, or more than two neighbors.
/// `visited`: is a set of nodes we have already visited, but only non-termination vertexes are
/// marked because termination nodes needs to be used several times.
/// returns a list of vertex id:s constituting the detected line in .windows(2) format
fn unwind_line(
    mut current: usize,
    next: Option<usize>,
    adjacency_map: &AHashMap<usize, SmallVec<[usize; 2]>>,
    termination_nodes: &AHashSet<usize>,
    visited: &mut AHashSet<usize>,
) -> Vec<usize> {
    let mut line = Vec::<usize>::default();
    let mut prev = Option::<usize>::None;

    if let Some(next) = next {
        //println!("pushed to line:{}", current);
        line.push(current);
        if !termination_nodes.contains(&current) {
            // don't mark termination nodes
            //println!("visited pushed {}", current);
            let _ = visited.insert(current);
        }
        prev = Some(current);
        current = next;
    }
    loop {
        if visited.contains(&current) || termination_nodes.contains(&current) {
            // we have gone round a closed shape
            /*println!(
                "detected visited (or termination) vertex:{} group:{:?}",
                current, line
            );*/
            //println!("pushed to line:{}", current);
            line.push(current);
            break;
        }
        //println!("visited pushed {}", current);
        let _ = visited.insert(current);
        //println!("pushed to line:{}", current);
        line.push(current);
        if let Some(neighbours) = adjacency_map.get(&current) {
            /*println!(
                "current:{}, neighbours:{:?} visited0:{}, visited1:{}",
                current,
                neighbours,
                visited.contains(&neighbours[0]),
                visited.contains(&neighbours[1])
            );*/
            if neighbours.len() != 2 {
                break;
            }
            // todo: if both options are open, pick the CCW one
            if !(visited.contains(&neighbours[0])
                || prev.is_some() && prev.unwrap() == neighbours[0])
            {
                /*println!(
                    "neighbours[1]={} was visited:{}",
                    &neighbours[1],
                    visited.contains(&neighbours[1])
                );
                println!("picking current = {}", neighbours[0]); //assert!(visited.contains(&neighbours[0]));
                */
                // neighbour 0 is unvisited
                prev = Some(current);
                current = neighbours[0];
            } else if !(visited.contains(&neighbours[1])
                || prev.is_some() && prev.unwrap() == neighbours[1])
            {
                /*println!(
                    "neighbours[0]={} was visited:{}",
                    &neighbours[0],
                    visited.contains(&neighbours[0])
                );
                println!("picking current = {}", neighbours[1]);
                */

                //assert!(!visited.contains(&neighbours[1]));
                // neighbour 1 is unvisited
                prev = Some(current);
                current = neighbours[1]
            } else {
                //println!("nowhere to go");
                break;
            }
        } else {
            break;
        }
    }
    //println!("Found a line:{:?}", line);
    line
}

/// Divides the `ìndices` into continuous shapes of vertex indices.
/// `ìndices` a list of unordered vertex indices in the .chunk(2) format. I.e [1,2,3,4,5] means
/// edges at [1,2], [3,4] & [4,5]
/// It will return lists of lists of continuous connected shapes. in the .windows(2) format.
/// If the shape describes a loop the first and last index will be the same.
/// TODO: Move this to the linestring crate
pub(crate) fn divide_into_shapes(indices: &[usize]) -> Vec<Vec<usize>> {
    let mut max_index = 0;
    // a vec containing identified shapes, and those are vertex indices in .windows(2) format
    let mut group_container = Vec::<Vec<usize>>::new();
    let mut min_index = 0_usize;
    // a map for vertex id to a list of adjacent vertices
    let mut adjacency_map = AHashMap::<usize, SmallVec<[usize; 2]>>::with_capacity(indices.len());

    indices.chunks(2).for_each(|chunk| {
        if chunk.len() == 2 {
            let i0 = chunk[0];
            let i1 = chunk[1];
            max_index = max_index.max(i0).max(i1);
            adjacency_map.entry(i0).or_default().push(i1);
            adjacency_map.entry(i1).or_default().push(i0);
        }
    });
    // these vertices are connected to 0 or >2 other vertices
    let mut termination_nodes = AHashSet::<usize>::with_capacity(max_index / 2);
    // these vertices are also connected to 0 or >2 other vertices, but will be continuously used/pop:ed.
    let mut candidate_nodes = BTreeMap::<usize, SmallVec<[usize; 2]>>::default();
    // Build the candidates and termination_nodes set
    // Build the candidates and termination_nodes set
    for v_id in 0..max_index + 1 {
        if let Some(neighbours) = adjacency_map.get(&v_id) {
            if neighbours.len() != 2 {
                let _ = termination_nodes.insert(v_id);
                let _ = candidate_nodes.insert(v_id, neighbours.clone());
            }
        } else {
            // vertices not even mentioned in the indices list
            let _ = termination_nodes.insert(v_id);
        }
    }
    // vertices that has already been marked as used
    let mut visited = AHashSet::<usize>::with_capacity(indices.len());

    /*println!(
            "adjacency_map:{:?}",
            adjacency_map
                .iter()
                .sorted_unstable_by(|a, b| a.0.cmp(b.0))
                .collect::<Vec<_>>()
        );
        println!("max_index:{:?}", max_index);
    */
    let mut current: usize = 0;

    // first stage: pop from the candidate list
    while !candidate_nodes.is_empty() {
        let mut next_vertex = Option::<usize>::None;
        while !candidate_nodes.is_empty() && next_vertex.is_none() {
            if let Some(mut current_entry) = candidate_nodes.first_entry() {
                {
                    current = *current_entry.key();
                    let array = current_entry.get_mut();
                    while !array.is_empty() {
                        let n_vertex = array.pop().unwrap();
                        if termination_nodes.contains(&n_vertex) || visited.contains(&n_vertex) {
                            continue;
                        } else {
                            next_vertex = Some(n_vertex);
                            break;
                        }
                    }
                }
                if current_entry.get().is_empty() {
                    let _ = current_entry.remove();
                }
            }
        }
        if next_vertex.is_some() {
            // next_vertex should now contain something
            group_container.push(unwind_line(
                current,
                next_vertex,
                &adjacency_map,
                &termination_nodes,
                &mut visited,
            ));
        }
    }

    /*println!(
            "stage two: visited:{} termination_nodes:{} total:{} max_index:{}",
            visited.len(),
            termination_nodes.len(),
            visited.len() + termination_nodes.len(),
            max_index
        );
        for group in &group_container {
            println!("group:{:?}", group);
        }
        println!(
            "visited:{:?}, len:{}",
            visited
                .iter()
                .sorted_unstable_by(|a, b| a.cmp(b))
                .collect::<Vec<_>>(),
            visited.len()
        );
        println!(
            "termination_nodes:{:?}, len:{}",
            termination_nodes
                .iter()
                .sorted_unstable_by(|a, b| a.cmp(b))
                .collect::<Vec<_>>(),
            termination_nodes.len()
        );
        assert!(candidate_nodes.is_empty());
    */
    // second stage, only loops remaining
    if visited.len() + termination_nodes.len() < max_index + 1 {
        'outer: loop {
            current = min_index;
            while visited.contains(&current) || termination_nodes.contains(&current) {
                current += 1;
                min_index = current;
                if current >= max_index {
                    // we probably have detected a loop again
                    current = min_index;
                    break;
                }
            }
            if current > max_index {
                if visited.len() + termination_nodes.len() >= max_index {
                    break 'outer;
                }
                // we did not find any isolated vertex, just pick one and start from there
                current = min_index;
                //only_loops_remain = true;
            }
            // `current` should now point to a vertex only connected to one, or more than 2 other vertexes
            // it could also point to a two-way connected vertex, but then all the rest are loops
            assert!(!visited.contains(&current));
            {
                // unravel the line at `current`
                let mut line = unwind_line(
                    current,
                    None,
                    &adjacency_map,
                    &termination_nodes,
                    &mut visited,
                );
                if line.len() > 1 {
                    line.push(*line.first().unwrap());
                    group_container.push(line);
                }
            }
            // current is now at the end or at a junction
            if visited.len() + termination_nodes.len() >= max_index {
                break;
            }
        }
    }
    group_container
}

// TODO:this re-creates the line strings just too many times
// TODO:rewrite this entire function
pub(crate) fn process_command<T: GenericVector3>(
    config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<(Vec<FFIVector3>, Vec<usize>, ConfigType), HallrError>
where
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
    HashableVector2: From<T::Vector2>,
{
    let epsilon: T::Scalar = config.get_mandatory_parsed_option("epsilon", None)?;
    //println!("rust: vertices.len():{}", vertices.len());
    //println!("rust: indices.len():{}", indices.len());
    //println!("rust: indices:{:?}", indices);
    //let result = divide_into_shapes(models[0].indices);
    //for group in result {
    //    println!("***group:{:?}", group);
    //}

    let simpify_3d = config.get_parsed_option("simplify_3d")?.unwrap_or(false);
    let mut output_vertices = Vec::<FFIVector3>::default();
    let mut output_indices = Vec::<usize>::default();

    if !models.is_empty() {
        let model = &models[0];
        output_vertices.reserve(model.vertices.len());
        output_indices.reserve(model.indices.len());

        let vertices = parse_input(&models[0])?;
        // todo: use another divide_into_shapes() method that uses the correct type 2d/3d
        if simpify_3d {
            // in 3d mode
            let mut v_3d_map =
                AHashMap::<(u32, u32, u32), usize>::with_capacity(model.indices.len());

            for line_string in divide_into_shapes(model.indices).into_iter().map(|line| {
                line.iter()
                    .map(|i| vertices[*i])
                    .collect::<LineString3<T>>()
            }) {
                //for line_string in line_string_set.set() {
                let simplified = line_string.simplify_rdp(epsilon);
                simplified.as_lines_iter().for_each(|line| {
                    let start = line.start;
                    let start_key = transmute_xyz_to_u32(&start);
                    //println!("testing {:?} as key {:?}", v2, v2_key);
                    let start_index = *v_3d_map.entry(start_key).or_insert_with(|| {
                        let new_index = output_vertices.len();
                        output_vertices.push(start.to());
                        //println!("i2 pushed ({},{},{}) as {}", v2.x(), v2.y(), v2.z(), new_index);
                        new_index
                    });
                    let end = line.end;
                    let end_key = transmute_xyz_to_u32(&end);
                    //println!("testing {:?} as key {:?}", v2, v2_key);
                    let end_index = *v_3d_map.entry(end_key).or_insert_with(|| {
                        let new_index = output_vertices.len();
                        output_vertices.push(end.to());
                        //println!("i2 pushed ({},{},{}) as {}", v2.x(), v2.y(), v2.z(), new_index);
                        new_index
                    });
                    output_indices.push(start_index);
                    output_indices.push(end_index);
                });
                //}
            }
        } else {
            // in 2d mode
            let mut v_2d_map = AHashMap::<(u32, u32), usize>::with_capacity(model.indices.len());
            for line_string in divide_into_shapes(model.indices).into_iter().map(|line| {
                line.iter()
                    .map(|i| vertices[*i])
                    .collect::<LineString3<T>>()
            }) {
                let simplified = line_string.copy_to_2d(Plane::XY).simplify_rdp(epsilon);
                simplified.line_iter().for_each(|line| {
                    let start = line.start;
                    let start_key = transmute_xy_to_u32(&start.to_3d(T::Scalar::ZERO));
                    //println!("testing {:?} as key {:?}", v2, v2_key);
                    let start_index = *v_2d_map.entry(start_key).or_insert_with(|| {
                        let new_index = output_vertices.len();
                        output_vertices.push(start.to_3d(T::Scalar::ZERO).to());
                        //println!("i2 pushed ({},{},{}) as {}", v2.x(), v2.y(), v2.z(), new_index);
                        new_index
                    });
                    let end = line.end;
                    let end_key = transmute_xy_to_u32(&end.to_3d(T::Scalar::ZERO));
                    //println!("testing {:?} as key {:?}", v2, v2_key);
                    let end_index = *v_2d_map.entry(end_key).or_insert_with(|| {
                        let new_index = output_vertices.len();
                        output_vertices.push(end.to_3d(T::Scalar::ZERO).to());
                        //println!("i2 pushed ({},{},{}) as {}", v2.x(), v2.y(), v2.z(), new_index);
                        new_index
                    });
                    output_indices.push(start_index);
                    output_indices.push(end_index);
                });
            }
        }
    }
    //println!("result vertices:{:?}", obj.vertices);
    //println!("result edges:{:?}", obj.lines.first());
    let mut config = ConfigType::new();
    let _ = config.insert("mesh.format".to_string(), "line_chunks".to_string());
    let _ = config.insert("REMOVE_DOUBLES".to_string(), "false".to_string());

    println!(
        "simplify_rdp operation returning {} vertices, {} indices",
        output_vertices.len(),
        output_indices.len()
    );
    Ok((output_vertices, output_indices, config))
}
