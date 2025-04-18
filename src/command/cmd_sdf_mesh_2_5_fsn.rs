// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use crate::{
    HallrError,
    command::{ConfigType, Model, Options, OwnedModel},
    ffi,
    ffi::FFIVector3,
};
use fast_surface_nets::{SurfaceNetsBuffer, ndshape::ConstShape, surface_nets};
use ilattice::{
    glam as iglam,
    prelude::{Extent, Vector2},
};
use linestring::linestring_3d::Plane;
use rayon::prelude::*;
use std::{borrow::Borrow, time};

// The un-padded chunk side, it will become 16*16*16
const UN_PADDED_CHUNK_SIDE: u32 = 14_u32;
type PaddedChunkShape = fast_surface_nets::ndshape::ConstShape3u32<
    { UN_PADDED_CHUNK_SIDE + 2 },
    { UN_PADDED_CHUNK_SIDE + 2 },
    { UN_PADDED_CHUNK_SIDE + 2 },
>;
const DEFAULT_SDF_VALUE: f32 = 999.0;
type Extent3i = Extent<iglam::IVec3>;

/// returns a list of type-converted vertices, a list of edges, and an AABB padded by radius
#[allow(clippy::type_complexity)]
fn parse_input(
    model: &Model<'_>,
    cmd_arg_radius_dimension: Plane,
) -> Result<(Vec<(iglam::Vec2, f32)>, Extent<iglam::Vec3A>), HallrError> {
    let zero = iglam::Vec3A::default();

    let mut aabb = {
        let vertex0 = model.vertices.first().ok_or_else(|| {
            HallrError::InvalidInputData("Input vertex list was empty".to_string())
        })?;
        Extent::from_min_and_shape(iglam::vec3a(vertex0.x, vertex0.y, vertex0.z), zero)
    };

    let vertices: Result<Vec<_>, HallrError> = model
        .vertices
        .iter()
        .map(|vertex| {
            if !vertex.x.is_finite() || !vertex.y.is_finite() || !vertex.z.is_finite() {
                Err(HallrError::InvalidInputData(format!(
                    "Only valid coordinates are allowed ({},{},{})",
                    vertex.x, vertex.y, vertex.z
                )))?
            } else {
                let (point2, radius) = match cmd_arg_radius_dimension {
                    Plane::YZ => (iglam::vec2(vertex.y, vertex.z), vertex.x.abs()),
                    Plane::XZ => (iglam::vec2(vertex.x, vertex.z), vertex.y.abs()),
                    Plane::XY => (iglam::vec2(vertex.x, vertex.y), vertex.z.abs()),
                };
                let v_aabb =
                    Extent::from_min_and_shape(iglam::vec3a(point2.x, point2.y, 0.0), zero)
                        .padded(radius);

                aabb = aabb.bound_union(&v_aabb);

                Ok((point2, radius))
            }
        })
        .collect();
    Ok((vertices?, aabb))
}

/// This is the sdf formula of a rounded cone (at origin)
///   vec2 q = vec2( length(p.xz), p.y );
///   float b = (r1-r2)/h;
///   float a = sqrt(1.0-b*b);
///   float k = dot(q,vec2(-b,a));
///   if( k < 0.0 ) return length(q) - r1;
///   if( k > a*h ) return length(q-vec2(0.0,h)) - r2;
///   return dot(q, vec2(a,b) ) - r1;
struct RoundedCone {
    r0: f32,
    r1: f32,
    h: f32,
    /// (r0-r1)/h
    b: f32,
    /// sqrt(1.0-b*b);
    a: f32,
    m: iglam::Affine3A,
}

/// Generate the data of a single chunk.
/// This code is run in a single thread
fn generate_and_process_sdf_chunk(
    un_padded_chunk_extent: Extent3i,
    rounded_cones: &[(RoundedCone, Extent3i)],
) -> Option<(iglam::Vec3A, SurfaceNetsBuffer)> {
    // the origin of this chunk, in voxel scale
    let padded_chunk_extent = un_padded_chunk_extent.padded(1);

    // filter out the edges that does not affect this chunk
    let filtered_cones: Vec<_> = rounded_cones
        .iter()
        .enumerate()
        .filter_map(|(index, sdf)| {
            if !padded_chunk_extent.intersection(sdf.1.borrow()).is_empty() {
                Some(index as u32)
            } else {
                None
            }
        })
        .collect();

    #[cfg(not(feature = "display_sdf_chunks"))]
    if filtered_cones.is_empty() {
        // no tubes intersected this chunk
        return None;
    }

    let mut array = { [DEFAULT_SDF_VALUE; PaddedChunkShape::SIZE as usize] };

    #[cfg(feature = "display_sdf_chunks")]
    // The corners of the un-padded chunk extent
    let corners: Vec<_> = un_padded_chunk_extent
        .corners3()
        .iter()
        .map(|p| p.as_vec3a())
        .collect();

    let mut some_neg_or_zero_found = false;
    let mut some_pos_found = false;

    for pwo in padded_chunk_extent.iter3() {
        let v = {
            let p = pwo - un_padded_chunk_extent.minimum + 1;
            &mut array[PaddedChunkShape::linearize([p.x as u32, p.y as u32, p.z as u32]) as usize]
        };
        // Point With Offset from the un-padded extent minimum
        let pwo = pwo.as_vec3a();

        #[cfg(feature = "display_sdf_chunks")]
        {
            // todo: this could probably be optimized with PaddedChunkShape::linearize(corner_pos)
            let mut x = *v;
            for c in corners.iter() {
                x = x.min(c.distance(pwo) - 1.);
            }
            *v = (*v).min(x);
        }
        for index in filtered_cones.iter() {
            let cone = &rounded_cones[*index as usize].0;
            let pwo = cone.m.transform_point3a(pwo);

            let q = iglam::Vec2::new(iglam::Vec2::new(pwo.x, pwo.z).length(), pwo.y);
            let k = q.dot(iglam::Vec2::new(-cone.b, cone.a));
            let new_v = if k < 0.0 {
                q.length() - cone.r0
            } else if k > cone.a * cone.h {
                (q - iglam::vec2(0.0, cone.h)).length() - cone.r1
            } else {
                q.dot(iglam::vec2(cone.a, cone.b)) - cone.r0
            };

            *v = (*v).min(new_v);
        }
        if *v > 0.0 {
            some_pos_found = true;
        } else {
            some_neg_or_zero_found = true;
        }
    }
    if some_pos_found && some_neg_or_zero_found {
        // A combination of positive and negative surfaces found - process this chunk
        let mut sn_buffer = SurfaceNetsBuffer::default();

        // do the voxel_size multiplication later, vertices pos. needs to match extent.
        surface_nets(
            &array,
            &PaddedChunkShape {},
            [0; 3],
            [UN_PADDED_CHUNK_SIDE + 1; 3],
            &mut sn_buffer,
        );

        if sn_buffer.positions.is_empty() {
            // No vertices were generated by this chunk, ignore it
            None
        } else {
            Some((padded_chunk_extent.minimum.as_vec3a(), sn_buffer))
        }
    } else {
        None
    }
}

#[allow(clippy::many_single_char_names)]
/// Build the chunk lattice and spawn off thread tasks for each chunk
fn build_voxel(
    divisions: f32,
    radius_multiplier: f32,
    vertices: Vec<(iglam::Vec2, f32)>,
    indices: &[usize],
    aabb: Extent<iglam::Vec3A>,
) -> Result<
    (
        f32, // voxel_size
        Vec<(iglam::Vec3A, SurfaceNetsBuffer)>,
    ),
    HallrError,
> {
    let zero = iglam::Vec3A::default();

    let max_dimension = {
        let dimensions = aabb.shape;
        dimensions.x.max(dimensions.y).max(dimensions.z)
    };

    let scale = divisions / max_dimension;

    let rounded_cones: Vec<(RoundedCone, Extent3i)> = indices
        .par_chunks_exact(2)
        .filter_map(|edge| {
            let (v0, r0) = vertices[edge[0]];
            let (v1, r1) = vertices[edge[1]];
            if r0 <= f32::EPSILON && r1 <= f32::EPSILON {
                return None;
            }

            let v0 = iglam::vec2(v0.x, v0.y) * scale;
            let r0 = r0 * scale * radius_multiplier;

            let v1 = iglam::vec2(v1.x, v1.y) * scale;
            let r1 = r1 * scale * radius_multiplier;

            let ex0 =
                Extent::<iglam::Vec3A>::from_min_and_shape(iglam::vec3a(v0.x, v0.y, 0.0), zero)
                    .padded(r0);
            let ex1 =
                Extent::<iglam::Vec3A>::from_min_and_shape(iglam::vec3a(v1.x, v1.y, 0.0), zero)
                    .padded(r1);
            // The AABB of the rounded cone intersected this chunk - keep it
            let v = v1 - v0;
            //let _c = v0 + v * 0.5; // center
            let h = v.length();
            let b = (r0 - r1) / h;
            let a = (1.0 - b * b).sqrt();
            // todo: this can't be correct and/or efficient
            let rotation = iglam::Mat3::from_rotation_z(v.angle_between(iglam::vec2(0.0, 1.0)));
            let translation = rotation.transform_point2(v0);
            let translation = -iglam::vec3(translation.x(), translation.y(), 0.0);
            let m = iglam::Affine3A::from_mat3_translation(rotation, translation);

            Some((
                RoundedCone { r0, r1, h, b, a, m },
                ex0.bound_union(&ex1).containing_integer_extent(),
            ))
        })
        .collect();

    let max_z_radius = aabb
        .minimum
        .z
        .abs()
        .max((aabb.minimum.z + aabb.shape.z).abs());
    let max_radius = scale * radius_multiplier * max_z_radius;
    let padding_voxels = max_radius * (UN_PADDED_CHUNK_SIDE as f32 / scale);
    //println!("max_z_radius:{}, max_radius:{}, padding_voxels:{}", max_z_radius, max_radius, padding_voxels);

    let chunks_extent =
        // pad with the radius + one voxel
        (aabb * (scale / (UN_PADDED_CHUNK_SIDE as f32)))
            .padded(padding_voxels)
            .containing_integer_extent();

    //println!("chunks_extent padded:{:?} scale:{} UN_PADDED_CHUNK_SIDE:{}", chunks_extent, scale, UN_PADDED_CHUNK_SIDE);
    let now = time::Instant::now();

    let sdf_chunks: Vec<_> = {
        let un_padded_chunk_shape = iglam::IVec3::splat(UN_PADDED_CHUNK_SIDE as i32);
        // Spawn off thread tasks creating and processing chunks.
        // Could also do:
        // (min.x..max.x).into_par_iter().flat_map(|x|
        //     (min.y..max.y).into_par_iter().flat_map(|y|
        //         (min.z..max.z).into_par_iter().map(|z| [x, y, z])))
        chunks_extent
            .iter3()
            .par_bridge()
            .filter_map(move |p| {
                let un_padded_chunk_extent =
                    Extent3i::from_min_and_shape(p * un_padded_chunk_shape, un_padded_chunk_shape);

                generate_and_process_sdf_chunk(un_padded_chunk_extent, &rounded_cones)
            })
            .collect()
    };
    println!(
        "process_chunks() duration: {:?} generated {} chunks",
        now.elapsed(),
        sdf_chunks.len()
    );

    Ok((1.0 / scale, sdf_chunks))
}

/// Build the return model
pub(crate) fn build_output_model(
    input_model: &Model<'_>,
    voxel_size: f32,
    mesh_buffers: Vec<(iglam::Vec3A, SurfaceNetsBuffer)>,
    cmd_arg_radius_axis: Plane,
    verbose: bool,
) -> Result<OwnedModel, HallrError> {
    let now = time::Instant::now();

    let (mut vertices, mut indices) = {
        // calculate the maximum required vertices & face capacity
        let (vertex_capacity, face_capacity) = mesh_buffers
            .iter()
            .fold((0_usize, 0_usize), |(v, f), chunk| {
                (v + chunk.1.positions.len(), f + chunk.1.indices.len())
            });
        if vertex_capacity >= u32::MAX as usize {
            return Err(HallrError::Overflow(format!(
                "Generated mesh contains too many vertices to be referenced by u32: {}. Reduce the resolution.",
                vertex_capacity
            )));
        }

        if face_capacity >= u32::MAX as usize {
            return Err(HallrError::Overflow(format!(
                "Generated mesh contains too many faces to be referenced by u32: {}. Reduce the resolution.",
                vertex_capacity
            )));
        }
        (
            Vec::with_capacity(vertex_capacity),
            Vec::with_capacity(face_capacity),
        )
    };

    if let Some(world_to_local) = input_model.get_world_to_local_transform()? {
        println!(
            "Rust: applying world-local transformation 1/{:?}",
            input_model.world_orientation
        );
        for (vertex_offset, mesh_buffer) in mesh_buffers.iter() {
            // each chunk starts counting vertices from zero
            let indices_offset = vertices.len() as u32;

            // vertices this far inside a chunk should (probably?) not be used outside this chunk.
            match cmd_arg_radius_axis {
                Plane::XY =>
                // Z axis is the radius dimension, no swap
                {
                    for pv in mesh_buffer.positions.iter() {
                        vertices.push(world_to_local(FFIVector3 {
                            x: (voxel_size * (pv[0] + vertex_offset.x)),
                            y: (voxel_size * (pv[1] + vertex_offset.y)),
                            z: (voxel_size * (pv[2] + vertex_offset.z)),
                        }));
                    }
                }
                Plane::XZ =>
                // Y axis is the radius dimension, swap X,Y,Z to X,Z,Y
                {
                    for pv in mesh_buffer.positions.iter() {
                        vertices.push(world_to_local(FFIVector3 {
                            x: (voxel_size * (pv[0] + vertex_offset.x)),
                            y: (voxel_size * (pv[2] + vertex_offset.z)),
                            z: (voxel_size * (pv[1] + vertex_offset.y)),
                        }));
                    }
                }
                Plane::YZ =>
                // X axis is the radius dimension, swap X,Y,Z to Y,Z,X
                {
                    for pv in mesh_buffer.positions.iter() {
                        vertices.push(world_to_local(FFIVector3 {
                            x: (voxel_size * (pv[2] + vertex_offset.z)),
                            y: (voxel_size * (pv[0] + vertex_offset.x)),
                            z: (voxel_size * (pv[1] + vertex_offset.y)),
                        }));
                    }
                }
            }
            for vertex_id in mesh_buffer.indices.iter() {
                indices.push((*vertex_id + indices_offset) as usize);
            }
        }
    } else {
        println!("Rust: *not* applying world-local transformation");
        for (vertex_offset, mesh_buffer) in mesh_buffers.iter() {
            // each chunk starts counting vertices from zero
            let indices_offset = vertices.len() as u32;

            // vertices this far inside a chunk should (probably?) not be used outside this chunk.
            match cmd_arg_radius_axis {
                Plane::XY =>
                // Z axis is the radius dimension, no swap
                {
                    for pv in mesh_buffer.positions.iter() {
                        vertices.push(FFIVector3 {
                            x: (voxel_size * (pv[0] + vertex_offset.x)),
                            y: (voxel_size * (pv[1] + vertex_offset.y)),
                            z: (voxel_size * (pv[2] + vertex_offset.z)),
                        });
                    }
                }
                Plane::XZ =>
                // Y axis is the radius dimension, swap X,Y,Z to X,Z,Y
                {
                    for pv in mesh_buffer.positions.iter() {
                        vertices.push(FFIVector3 {
                            x: (voxel_size * (pv[0] + vertex_offset.x)),
                            y: (voxel_size * (pv[2] + vertex_offset.z)),
                            z: (voxel_size * (pv[1] + vertex_offset.y)),
                        });
                    }
                }
                Plane::YZ =>
                // X axis is the radius dimension, swap X,Y,Z to Y,Z,X
                {
                    for pv in mesh_buffer.positions.iter() {
                        vertices.push(FFIVector3 {
                            x: (voxel_size * (pv[2] + vertex_offset.z)),
                            y: (voxel_size * (pv[0] + vertex_offset.x)),
                            z: (voxel_size * (pv[1] + vertex_offset.y)),
                        });
                    }
                }
            }
            for vertex_id in mesh_buffer.indices.iter() {
                indices.push((*vertex_id + indices_offset) as usize);
            }
        }
    }

    if verbose {
        println!(
            "Vertex return model packaging duration: {:?}",
            now.elapsed()
        );
    }
    Ok(OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        //name: pb_model_name,
        vertices,
        indices,
    })
}

/// Run the voronoi_mesh command
pub(crate) fn process_command(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    if models.is_empty() {
        return Err(HallrError::InvalidInputData(
            "This operation requires ome input model".to_string(),
        ));
    }

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::LineChunks)?;

    if models.len() > 1 {
        return Err(HallrError::InvalidInputData(
            "This operation only supports one model as input".to_string(),
        ));
    }

    let cmd_arg_sdf_divisions: f32 =
        input_config.get_mandatory_parsed_option("SDF_DIVISIONS", None)?;
    if !(9.9..600.1).contains(&cmd_arg_sdf_divisions) {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of SDF_DIVISIONS is [{}..{}[% :({})",
            10, 600, cmd_arg_sdf_divisions
        )));
    }

    let cmd_arg_sdf_radius_multiplier =
        input_config.get_mandatory_parsed_option::<f32>("SDF_RADIUS_MULTIPLIER", None)?;

    // we already tested a_command.models.len()
    let input_model = &models[0];

    println!("model.vertices:{:?}, ", input_model.vertices.len());

    let plane = Plane::XY;
    let (vertices, aabb) = parse_input(input_model, plane)?;
    let (voxel_size, mesh) = build_voxel(
        cmd_arg_sdf_divisions,
        cmd_arg_sdf_radius_multiplier,
        vertices,
        input_model.indices,
        aabb,
    )?;

    let output_model = build_output_model(input_model, voxel_size, mesh, plane, false)?;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    if let Some(mv) = input_config.get_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }
    println!(
        "sdf mesh 2.5d operation returning {} vertices, {} indices",
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
