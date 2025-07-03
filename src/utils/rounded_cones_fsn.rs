// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    HallrError,
    command::{Model, OwnedModel},
    ffi::FFIVector3,
};
use fast_surface_nets::{SurfaceNetsBuffer, ndshape::ConstShape};
use ilattice::{glam as iglam, prelude::Extent};
use rayon::{iter::ParallelIterator, prelude::IntoParallelIterator};
use std::time;
use vector_traits::{
    glam,
    prelude::{Aabb3, GenericVector3},
};

type Extent3i = Extent<iglam::IVec3>;
// The un-padded chunk side, it will become 16*16*16
pub const UN_PADDED_CHUNK_SIDE: u32 = 14_u32;
pub type PaddedChunkShape = fast_surface_nets::ndshape::ConstShape3u32<
    { UN_PADDED_CHUNK_SIDE + 2 },
    { UN_PADDED_CHUNK_SIDE + 2 },
    { UN_PADDED_CHUNK_SIDE + 2 },
>;

pub const DEFAULT_SDF_VALUE: f32 = 999.0;

/// This is the sdf formula of a round cone (tapered capsule)
struct RoundCone {
    r0: f32,              // Radius at start
    r1: f32,              // Radius at end
    center0: glam::Vec3A, // Center of first sphere

    // Pre-calculated constants for optimization
    ba: glam::Vec3A, // Vector from center0 to center1
    l2: f32,         // Squared length of ba
    rr: f32,         // r0 - r1
    rr3: f32,        // rr^3 (sign(rr) * rr * rr)
    a2: f32,         // l2 - rr*rr
    il2: f32,        // 1.0 / l2
}

impl RoundCone {
    fn new(center0: iglam::Vec3A, center1: iglam::Vec3A, r0: f32, r1: f32) -> Self {
        let ba = center1 - center0;
        let l2 = ba.length_squared();
        let rr = r0 - r1;

        RoundCone {
            r0,
            r1,
            center0: glam::vec3a(center0.x, center0.y, center0.z),
            ba: glam::vec3a(ba.x, ba.y, ba.z),
            l2,
            rr,
            rr3: rr.signum() * rr * rr,
            a2: l2 - rr * rr,
            il2: 1.0 / l2,
        }
    }
}

// Helper function equivalent to GLSL's dot2 (dot product with itself)
#[inline(always)]
fn dot2(v: glam::Vec3A) -> f32 {
    v.dot(v)
}

#[inline(always)]
// source : https://iquilezles.org/articles/distfunctions/
fn sdf_round_cone(p: glam::Vec3A, capsule: &RoundCone) -> f32 {
    // Handle degenerate case where centers are the same
    if capsule.l2 <= f32::EPSILON * f32::EPSILON {
        return (p - capsule.center0).length() - capsule.r0;
    }

    // sampling dependent computations
    let pa = p - capsule.center0;
    let y = pa.dot(capsule.ba);
    let z = y - capsule.l2;
    let x2 = dot2(pa * capsule.l2 - capsule.ba * y);
    let y2 = y * y * capsule.l2;
    let z2 = z * z * capsule.l2;

    let k = capsule.rr3 * x2;

    if z.signum() * capsule.a2 * z2 > k {
        return (x2 + z2).sqrt() * capsule.il2 - capsule.r1;
    }
    if y.signum() * capsule.a2 * y2 < k {
        return (x2 + y2).sqrt() * capsule.il2 - capsule.r0;
    }

    ((x2 * capsule.a2 * capsule.il2).sqrt() + y * capsule.rr) * capsule.il2 - capsule.r0
}

/// Build the chunk lattice and spawn off threaded tasks for each chunk
pub(crate) fn build_round_cones_voxel_mesh<I>(
    divisions: f32,
    edges: I,
    edges_aabb: <glam::Vec3 as GenericVector3>::Aabb,
) -> Result<
    (
        f32, // voxel_size
        Vec<(glam::Vec3, SurfaceNetsBuffer)>,
    ),
    HallrError,
>
where
    I: IntoParallelIterator<Item = (glam::Vec4, glam::Vec4)>,
{
    let edges_aabb = {
        let (min, _, shape) = edges_aabb.extents();
        Extent::<iglam::Vec3A>::from_min_and_shape(
            iglam::vec3a(min.x, min.y, min.z),
            iglam::vec3a(shape.x, shape.y, shape.z),
        )
    };

    let max_dimension = {
        let dimensions = edges_aabb.shape;
        dimensions.x.max(dimensions.y).max(dimensions.z)
    };

    let scale = divisions / max_dimension;

    #[cfg(feature = "display_sdf_chunks")]
    println!(
        "display_sdf_chunks is enabled, input aabb : {edges_aabb:?}, divisions: {divisions:?}, scale: {scale:?}"
    );
    let round_cones: Vec<(RoundCone, Extent3i)> = edges
        .into_par_iter()
        .filter_map(|edge| {
            let (v0, v1) = edge;
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
                RoundCone::new(center0, center1, r0, r1),
                ex0.bound_union(&ex1).containing_integer_extent(),
            ))
        })
        .collect();

    let padding_voxels = 1.0;
    #[cfg(feature = "display_sdf_chunks")]
    println!(" padding_voxels:{padding_voxels}");

    let chunks_extent =
        // pad with the radius + one voxel
        (edges_aabb * (scale / (UN_PADDED_CHUNK_SIDE as f32)))
            .padded(padding_voxels)
            .containing_integer_extent();

    #[cfg(feature = "display_sdf_chunks")]
    println!(
        "chunks_extent {chunks_extent:?} scale:{scale} UN_PADDED_CHUNK_SIDE:{UN_PADDED_CHUNK_SIDE}"
    );
    let now = time::Instant::now();

    let sdf_chunks: Vec<_> = {
        let un_padded_chunk_shape = iglam::IVec3::splat(UN_PADDED_CHUNK_SIDE as i32);
        chunks_extent
            .par_iter3()
            .filter_map(move |p| {
                let un_padded_chunk_extent =
                    Extent3i::from_min_and_shape(p * un_padded_chunk_shape, un_padded_chunk_shape);

                generate_and_process_sdf_chunk(un_padded_chunk_extent, &round_cones)
            })
            .collect()
    };
    println!(
        "Rust: process_chunks() duration: {:?} generated {} chunks",
        now.elapsed(),
        sdf_chunks.len()
    );

    Ok((1.0 / scale, sdf_chunks))
}

/// Generate the data of a single chunk.
/// This code is run in a parallel
fn generate_and_process_sdf_chunk(
    un_padded_chunk_extent: Extent3i,
    round_cones: &[(RoundCone, Extent3i)],
) -> Option<(glam::Vec3, SurfaceNetsBuffer)> {
    // the origin of this chunk, in voxel scale
    let padded_chunk_extent = un_padded_chunk_extent.padded(1);

    // filter out the edges that does not affect this chunk
    let filtered_capsules: Vec<_> = round_cones
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

    let mut array = { [DEFAULT_SDF_VALUE; PaddedChunkShape::SIZE as usize] };

    #[cfg(feature = "display_sdf_chunks")]
    // The corners of the un-padded chunk extent
    let corners: Vec<_> = un_padded_chunk_extent
        .corners3()
        .iter()
        .map(|p| glam::vec3a(p.x as f32, p.y as f32, p.z as f32))
        .collect();

    let mut some_neg_or_zero_found = false;
    let mut some_pos_found = false;

    for pwo in padded_chunk_extent.iter3() {
        let v = {
            let p = pwo - un_padded_chunk_extent.minimum + 1;
            &mut array[PaddedChunkShape::linearize([p.x as u32, p.y as u32, p.z as u32]) as usize]
        };
        // Point With Offset from the un-padded extent minimum
        let pwo = glam::vec3a(pwo.x as f32, pwo.y as f32, pwo.z as f32);

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
            let capsule = &round_cones[*index as usize].0;

            *v = (*v).min(sdf_round_cone(pwo, capsule));
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
        //fast_surface_nets::surface_nets_with_config::<fast_surface_nets::NoNormals, _, _>(
        fast_surface_nets::surface_nets(
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
            let min = padded_chunk_extent.minimum;
            Some((
                glam::vec3(min.x as f32, min.y as f32, min.z as f32),
                sn_buffer,
            ))
        }
    } else {
        None
    }
}

/// Build the return model
pub(crate) fn build_output_model(
    input_model: Option<&Model<'_>>,
    voxel_size: f32,
    mesh_buffers: Vec<(glam::Vec3, SurfaceNetsBuffer)>,
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

    if let Some(world_to_local) =
        input_model.and_then(|im| im.get_world_to_local_transform().transpose())
    {
        let world_to_local = world_to_local?;
        println!("Rust: applying world-local transformation",);
        for (vertex_offset, mesh_buffer) in mesh_buffers.iter() {
            // each chunk starts counting vertices from zero
            let indices_offset = vertices.len() as u32;

            // vertices this far inside a chunk should (probably?) not be used outside this chunk.
            for pv in mesh_buffer.positions.iter() {
                vertices.push(world_to_local(FFIVector3 {
                    x: (voxel_size * (pv[0] + vertex_offset.x)),
                    y: (voxel_size * (pv[1] + vertex_offset.y)),
                    z: (voxel_size * (pv[2] + vertex_offset.z)),
                }));
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
    }

    if verbose {
        println!(
            "Rust: Vertex return model packaging duration: {:?}",
            now.elapsed()
        );
    }
    Ok(OwnedModel {
        world_orientation: OwnedModel::identity_matrix(),
        vertices,
        indices,
    })
}
