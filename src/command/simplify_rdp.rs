use super::{ConfigType, Model, Options};
use crate::{geo::HashableVector2, prelude::*};
use hronn::prelude::*;
use linestring::linestring_3d::Plane;
use vector_traits::{
    num_traits::AsPrimitive, GenericScalar, GenericVector2, GenericVector3, HasXY, HasXYZ,
};

/// converts to a private, comparable and hashable format
/// only use this for floats that are f32::is_finite()
/// This will only work for floats that's identical in every bit.
/// The z coordinate will not be used because it might be slightly different
/// depending on how it was calculated. Not using z will also make the calculations faster.
#[inline(always)]
fn transmute_xy_to_u32<T: HasXYZ>(a: &T) -> (u32, u32) {
    let x: f32 = a.x().as_();
    let y: f32 = a.y().as_();
    (x.to_bits(), y.to_bits())
}

/// converts to a private, comparable and hashable format
/// only use this for floats that are f32::is_finite()
/// This will only work for floats that's identical in every bit.
fn transmute_xyz_to_u32<T: HasXYZ>(a: &T) -> (u32, u32, u32) {
    let x: f32 = a.x().as_();
    let y: f32 = a.y().as_();
    let z: f32 = a.z().as_();
    (x.to_bits(), y.to_bits(), z.to_bits())
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

/// reformat the input into a edge set and converted vertices.
#[allow(clippy::type_complexity)]
fn parse_input<T: GenericVector3>(
    model: &Model<'_>,
) -> Result<(ahash::AHashSet<(usize, usize)>, Vec<T>), HallrError>
where
    FFIVector3: ConvertTo<T>,
{
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

    Ok((edge_set, converted_vertices))
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
    let epsilon: T::Scalar = config.get_mandatory_parsed_option("epsilon")?;
    //println!("rust: vertices.len():{}", vertices.len());
    //println!("rust: indices.len():{}", indices.len());
    //println!("rust: indices:{:?}", indices);
    let simpify_3d = config.get_parsed_option("simplify_3d")?.unwrap_or(false);
    let mut output_vertices = Vec::<FFIVector3>::default();
    let mut output_indices = Vec::<usize>::default();

    if !models.is_empty() {
        let model = &models[0];
        output_vertices.reserve(model.vertices.len());
        output_indices.reserve(model.indices.len());

        let (edge_set, vertices) = parse_input(&models[0])?;
        // todo: use another divide_into_shapes() method that uses the correct type 2d/3d
        if simpify_3d {
            // in 3d mode
            let mut v_3d_map =
                ahash::AHashMap::<(u32, u32, u32), usize>::with_capacity(model.indices.len());

            let lines = centerline::divide_into_shapes(edge_set, vertices)?;
            for line_string_set in lines {
                for line_string in line_string_set.set() {
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
                }
            }
        } else {
            // in 2d mode
            let mut v_2d_map =
                ahash::AHashMap::<(u32, u32), usize>::with_capacity(model.indices.len());
            let lines = centerline::divide_into_shapes(edge_set, vertices)?;
            for line_string_set in lines {
                for line_string in line_string_set.set() {
                    let simplified = line_string.copy_to_2d(Plane::XY).simplify_rdp(epsilon);
                    simplified.iter().for_each(|line| {
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
