// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{OwnedModel, cmd_sdf_mesh_2_5_fsn::UN_PADDED_CHUNK_SIDE},
    ffi::FFIVector3,
};
use fast_surface_nets::{SurfaceNetsBuffer, ndshape::ConstShape, surface_nets};
use ilattice::{glam as iglam, prelude::Extent};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use std::time;
use vector_traits::glam;

type Extent3i = Extent<iglam::IVec3>;

/// This is the sdf formula of a tapered capsule (at origin)
struct TaperedCapsule {
    r0: f32,               // Radius at start
    r1: f32,               // Radius at end
    h: f32,                // Length of the capsule
    center0: iglam::Vec3A, // Center of first sphere
    center1: iglam::Vec3A, // Center of second sphere
}

fn sdf_tapered_capsule(p: iglam::Vec3A, capsule: &TaperedCapsule) -> f32 {
    // Vector from center0 to p
    let ba = capsule.center1 - capsule.center0;
    let pa = p - capsule.center0;
    let _pb = p - capsule.center1;

    // Handle degenerate case
    if capsule.h <= f32::EPSILON {
        return (p - capsule.center0).length() - capsule.r0;
    }

    // Normalized axis
    let axis = ba / capsule.h;

    // Projection of pa onto axis
    let t = pa.dot(axis);

    // Project onto the line segment
    let t_clamped = t.clamp(0.0, capsule.h);

    // Compute the point on the segment that's closest to p
    let closest_on_segment = capsule.center0 + axis * t_clamped;

    // Distance from p to the closest point on the segment
    let d = (p - closest_on_segment).length();

    // Interpolate radius at this point
    let radius = capsule.r0 + (capsule.r1 - capsule.r0) * (t_clamped / capsule.h);

    // SDF value
    d - radius
}

#[allow(clippy::many_single_char_names)]
/// Build the chunk lattice and spawn off thread tasks for each chunk
pub(super) fn build_voxel(
    divisions: f32,
    edges: Vec<[glam::Vec4; 2]>,
    aabb: Extent<iglam::Vec3A>,
) -> Result<
    (
        f32, // voxel_size
        Vec<(iglam::Vec3A, SurfaceNetsBuffer)>,
    ),
    HallrError,
> {
    let max_dimension = {
        let dimensions = aabb.shape;
        dimensions.x.max(dimensions.y).max(dimensions.z)
    };

    let scale = divisions / max_dimension;

    let tapered_capsules: Vec<(TaperedCapsule, Extent3i)> = edges
        .par_iter()
        .filter_map(|edge| {
            let [v0, v1] = edge;
            let r0 = v0.w;
            let r1 = v1.w;

            if r0 <= f32::EPSILON && r1 <= f32::EPSILON {
                return None;
            }

            // Convert to 3D points with proper scaling
            let center0 = iglam::vec3a(v0.x, v0.y, v0.z) * scale;
            let r0 = r0 * scale;

            let center1 = iglam::vec3a(v1.x, v1.y, v1.z) * scale;
            let r1 = r1 * scale;

            // Calculate capsule length
            let h = (center1 - center0).length();

            // Skip degenerate capsules
            if h <= f32::EPSILON {
                return None;
            }

            // Create bounding boxes
            let ex0 = Extent::<iglam::Vec3A>::from_min_and_shape(
                iglam::vec3a(center0.x, center0.y, center0.z),
                iglam::Vec3A::ZERO,
            )
            .padded(r0);
            let ex1 = Extent::<iglam::Vec3A>::from_min_and_shape(
                iglam::vec3a(center1.x, center1.y, center1.z),
                iglam::Vec3A::ZERO,
            )
            .padded(r1);

            Some((
                TaperedCapsule {
                    r0,
                    r1,
                    h,
                    center0,
                    center1,
                },
                ex0.bound_union(&ex1).containing_integer_extent(),
            ))
        })
        .collect();

    let max_z_radius = aabb
        .minimum
        .z
        .abs()
        .max((aabb.minimum.z + aabb.shape.z).abs());
    let max_radius = scale * max_z_radius;
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
        chunks_extent
            .iter3()
            .par_bridge()
            .filter_map(move |p| {
                let un_padded_chunk_extent =
                    Extent3i::from_min_and_shape(p * un_padded_chunk_shape, un_padded_chunk_shape);

                generate_and_process_sdf_chunk(un_padded_chunk_extent, &tapered_capsules)
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

/// Generate the data of a single chunk.
/// This code is run in a single thread
fn generate_and_process_sdf_chunk(
    un_padded_chunk_extent: Extent3i,
    tapered_capsules: &[(TaperedCapsule, Extent3i)],
) -> Option<(iglam::Vec3A, SurfaceNetsBuffer)> {
    // the origin of this chunk, in voxel scale
    let padded_chunk_extent = un_padded_chunk_extent.padded(1);

    // filter out the edges that does not affect this chunk
    let filtered_capsules: Vec<_> = tapered_capsules
        .iter()
        .enumerate()
        .filter_map(|(index, sdf)| {
            if !padded_chunk_extent.intersection(&sdf.1).is_empty() {
                Some(index as u32)
            } else {
                None
            }
        })
        .collect();

    #[cfg(not(feature = "display_sdf_chunks"))]
    if filtered_capsules.is_empty() {
        // no tubes intersected this chunk
        return None;
    }

    let mut array = {
        [crate::command::cmd_sdf_mesh_2_5_fsn::DEFAULT_SDF_VALUE;
            crate::command::cmd_sdf_mesh_2_5_fsn::PaddedChunkShape::SIZE as usize]
    };

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
            &mut array[crate::command::cmd_sdf_mesh_2_5_fsn::PaddedChunkShape::linearize([
                p.x as u32, p.y as u32, p.z as u32,
            ]) as usize]
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
        for index in filtered_capsules.iter() {
            let capsule = &tapered_capsules[*index as usize].0;

            *v = (*v).min(sdf_tapered_capsule(pwo, capsule));
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
            &crate::command::cmd_sdf_mesh_2_5_fsn::PaddedChunkShape {},
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

/// Build the return model
pub(crate) fn build_output_model(
    voxel_size: f32,
    mesh_buffers: Vec<(iglam::Vec3A, SurfaceNetsBuffer)>,
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
                "Generated mesh contains too many vertices to be referenced by u32: {vertex_capacity}. Reduce the resolution.",
            )));
        }

        if face_capacity >= u32::MAX as usize {
            return Err(HallrError::Overflow(format!(
                "Generated mesh contains too many faces to be referenced by u32: {vertex_capacity}. Reduce the resolution.",
            )));
        }
        (
            Vec::with_capacity(vertex_capacity),
            Vec::with_capacity(face_capacity),
        )
    };

    println!("Rust: *not* applying world-local transformation");
    for (vertex_offset, mesh_buffer) in mesh_buffers.iter() {
        // each chunk starts counting vertices from zero
        let indices_offset = vertices.len() as u32;

        // vertices this far inside a chunk should (probably?) not be used outside this chunk.
        for pv in mesh_buffer.positions.iter() {
            vertices.push(FFIVector3 {
                x: (voxel_size * (pv[0] + vertex_offset.x)),
                y: (voxel_size * (pv[1] + vertex_offset.y)),
                z: (voxel_size * (pv[2] + vertex_offset.z)),
            });
        }
        for vertex_id in mesh_buffer.indices.iter() {
            indices.push((*vertex_id + indices_offset) as usize);
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
        vertices,
        indices,
    })
}
