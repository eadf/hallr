// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use crate::{
    HallrError,
    command::{ConfigType, Model, Options, OwnedModel},
    ffi,
    ffi::FFIVector3,
};
use fast_surface_nets::{SurfaceNetsBuffer, ndshape::ConstShape};
use ilattice::{glam as iglam, prelude::Extent};
use rayon::prelude::*;
use std::time;
use vector_traits::glam;

// The un-padded chunk side, it will become 16*16*16
const UN_PADDED_CHUNK_SIDE: u32 = 14_u32;
type PaddedChunkShape = fast_surface_nets::ndshape::ConstShape3u32<
    { UN_PADDED_CHUNK_SIDE + 2 },
    { UN_PADDED_CHUNK_SIDE + 2 },
    { UN_PADDED_CHUNK_SIDE + 2 },
>;
const DEFAULT_SDF_VALUE: f32 = 999.0;
type Extent3i = Extent<iglam::IVec3>;

/// returns an AABB (not padded by radius)
fn parse_input(model: &Model<'_>) -> Result<Extent<iglam::Vec3A>, HallrError> {
    let zero = iglam::Vec3A::ZERO;
    let mut aabb = {
        let vertex0 = model.vertices.first().ok_or_else(|| {
            HallrError::InvalidInputData("Input vertex list was empty".to_string())
        })?;
        Extent::from_min_and_shape(iglam::vec3a(vertex0.x, vertex0.y, vertex0.z), zero)
    };

    for vertex in model.vertices.iter() {
        if !vertex.is_finite() {
            Err(HallrError::InvalidInputData(format!(
                "Only finite coordinates are allowed ({},{},{})",
                vertex.x, vertex.y, vertex.z
            )))?
        } else {
            let point = iglam::vec3a(vertex.x, vertex.y, vertex.z);
            let v_aabb = Extent::from_min_and_shape(point, zero);
            aabb = aabb.bound_union(&v_aabb);
        }
    }

    Ok(aabb)
}

/// Build the chunk lattice and spawn off thread tasks for each chunk
fn build_voxel(
    radius_multiplier: f32,
    divisions: f32,
    vertices: &[FFIVector3],
    indices: &[u32],
    unpadded_aabb: Extent<iglam::Vec3A>,
    verbose: bool,
) -> Result<
    (
        f32, // voxel_size
        Vec<SurfaceNetsBuffer>,
    ),
    HallrError,
> {
    let max_dimension = {
        let dimensions = unpadded_aabb.shape;
        dimensions.x.max(dimensions.y).max(dimensions.z)
    };

    let radius = max_dimension * radius_multiplier; // unscaled
    let scale = divisions / max_dimension;
    // Add the radius padding around the aabb
    let aabb = unpadded_aabb.padded(radius);

    if verbose {
        println!(
            "Rust: Voxelizing using tube radius. {radius} = {max_dimension}*{radius_multiplier}*{scale}"
        );

        println!(
            "Rust: Voxelizing using divisions = {divisions}, max dimension = {max_dimension}, scale factor={scale} (max_dimension*scale={})",
            max_dimension * scale
        );
        println!();
    }
    let vertices: Vec<_> = vertices
        .iter()
        .map(|v| glam::Vec3A::new(v.x, v.y, v.z) * scale)
        .collect();

    let chunks_extent = {
        // pad with the radius + one voxel
        (aabb * (scale / (UN_PADDED_CHUNK_SIDE as f32)))
            .padded(1.0 / (UN_PADDED_CHUNK_SIDE as f32))
            .containing_integer_extent()
    };

    let now = time::Instant::now();

    let sdf_chunks: Vec<_> = {
        let radius = radius * scale;
        let unpadded_chunk_shape = iglam::IVec3::splat(UN_PADDED_CHUNK_SIDE as i32);
        // Spawn off thread tasks creating and processing chunks.
        chunks_extent
            .par_iter3()
            .filter_map(move |p| {
                let unpadded_chunk_extent =
                    Extent3i::from_min_and_shape(p * unpadded_chunk_shape, unpadded_chunk_shape);

                generate_and_process_sdf_chunk(unpadded_chunk_extent, &vertices, indices, radius)
            })
            .collect()
    };

    if verbose {
        println!(
            "Rust: process_chunks() duration: {:?} generated {} chunks",
            now.elapsed(),
            sdf_chunks.len()
        );
    }

    Ok((1.0 / scale, sdf_chunks))
}

#[inline(always)]
fn extent_from_min_and_lub(min: glam::Vec3A, lub: glam::Vec3A) -> Extent<iglam::Vec3A> {
    Extent::from_min_and_lub(
        iglam::vec3a(min.x, min.y, min.z),
        iglam::vec3a(lub.x, lub.y, lub.z),
    )
}

/// Generate the data of a single chunk
fn generate_and_process_sdf_chunk(
    unpadded_chunk_extent: Extent3i,
    vertices: &[glam::Vec3A],
    indices: &[u32],
    thickness: f32,
) -> Option<SurfaceNetsBuffer> {
    let thickness_v = glam::Vec3A::splat(thickness);
    // the origin of this chunk, in voxel scale
    let padded_chunk_extent = unpadded_chunk_extent.padded(1);

    // filter out the edges that does not affect this chunk
    let filtered_edges: Vec<_> = indices
        .par_chunks_exact(2)
        .filter_map(|edge| {
            let (e0, e1) = (edge[0], edge[1]);
            let v0 = vertices[e0 as usize];
            let v1 = vertices[e1 as usize];

            let tube_extent =
                extent_from_min_and_lub(v0.min(v1) - thickness_v, v0.max(v1) + thickness_v)
                    .containing_integer_extent();
            if !padded_chunk_extent.intersection(&tube_extent).is_empty() {
                // The AABB of the edge tube intersected this chunk - keep it
                Some((e0, e1))
            } else {
                None
            }
        })
        .collect();

    #[cfg(not(feature = "display_sdf_chunks"))]
    if filtered_edges.is_empty() {
        // no tubes intersected this chunk
        return None;
    }

    let mut array = { [DEFAULT_SDF_VALUE; PaddedChunkShape::SIZE as usize] };

    #[cfg(feature = "display_sdf_chunks")]
    // The corners of the un-padded chunk extent
    let corners: Vec<_> = unpadded_chunk_extent
        .corners3()
        .iter()
        .map(|p| p.as_vec3a())
        .collect();

    let mut some_neg_or_zero_found = false;
    let mut some_pos_found = false;

    // Point With Offset from the un-padded extent minimum
    for pwo in padded_chunk_extent.iter3() {
        let v = {
            let p = pwo - unpadded_chunk_extent.minimum + 1;
            &mut array[PaddedChunkShape::linearize([p.x as u32, p.y as u32, p.z as u32]) as usize]
        };

        #[cfg(feature = "display_sdf_chunks")]
        {
            let mut x = *v;
            for c in corners.iter() {
                x = x.min(c.distance(pwo.as_vec3a()) - 1.);
            }
            *v = (*v).min(x);
        }
        for (from_v, to_v) in filtered_edges
            .iter()
            .map(|(e0, e1)| (vertices[*e0 as usize], vertices[*e1 as usize]))
        {
            // This is the sdf formula of a capsule
            let pa = glam::vec3a(pwo.x as f32, pwo.y as f32, pwo.z as f32) - from_v;
            let ba = to_v - from_v;
            let t = pa.dot(ba) / ba.dot(ba);
            let h = t.clamp(0.0, 1.0);
            *v = (*v).min((pa - (ba * h)).length() - thickness);
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
        //fast_surface_nets::surface_nets_with_config::<fast_surface_nets::NoNormals, _, _,>(
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
            // Offset vertices to world coordinates
            let world_offset = padded_chunk_extent.minimum;
            for pos in sn_buffer.positions.iter_mut() {
                pos[0] += world_offset.x as f32;
                pos[1] += world_offset.y as f32;
                pos[2] += world_offset.z as f32;
            }

            Some(sn_buffer)
        }
    } else {
        None
    }
}

/// Build the return model
pub(crate) fn build_output_model(
    voxel_size: f32,
    mesh_buffers: Vec<SurfaceNetsBuffer>,
    world_to_local: Option<impl Fn(FFIVector3) -> FFIVector3>,
    verbose: bool,
) -> Result<OwnedModel, HallrError> {
    let now = time::Instant::now();

    let (mut vertices, mut indices) = {
        // calculate the maximum required vertices & facec capacity
        let (vertex_capacity, face_capacity) = mesh_buffers.iter().fold((0, 0), |(v, f), chunk| {
            (v + chunk.positions.len(), f + chunk.indices.len())
        });
        if vertex_capacity >= u32::MAX as usize {
            return Err(HallrError::Overflow(format!(
                "Generated mesh contains too many vertices to be referenced by u32: {vertex_capacity}. Reduce the resolution."
            )));
        }

        if face_capacity >= u32::MAX as usize {
            return Err(HallrError::Overflow(format!(
                "Generated mesh contains too many faces to be referenced by u32: {vertex_capacity}. Reduce the resolution."
            )));
        }
        (
            Vec::with_capacity(vertex_capacity),
            Vec::with_capacity(face_capacity),
        )
    };
    if let Some(world_to_local) = world_to_local {
        for mesh_buffer in mesh_buffers.iter() {
            // each chunk starts counting vertices from zero
            let indices_offset = vertices.len() as u32;

            // vertices this far inside a chunk should (probably?) not be used outside this chunk.

            for pv in mesh_buffer.positions.iter() {
                vertices.push(world_to_local(FFIVector3 {
                    x: (voxel_size * (pv[0])),
                    y: (voxel_size * (pv[1])),
                    z: (voxel_size * (pv[2])),
                }));
            }

            for vertex_id in mesh_buffer.indices.iter() {
                indices.push(*vertex_id + indices_offset);
            }
        }
    } else {
        for mesh_buffer in mesh_buffers.iter() {
            // each chunk starts counting vertices from zero
            let indices_offset = vertices.len() as u32;

            // vertices this far inside a chunk should (probably?) not be used outside this chunk.

            for pv in mesh_buffer.positions.iter() {
                vertices.push(FFIVector3 {
                    x: (voxel_size * (pv[0])),
                    y: (voxel_size * (pv[1])),
                    z: (voxel_size * (pv[2])),
                });
            }

            for vertex_id in mesh_buffer.indices.iter() {
                indices.push(*vertex_id + indices_offset);
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

/// Run the sdf_mesh command
pub(crate) fn process_command(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
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

    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Edges)?;

    let cmd_arg_sdf_radius_multiplier =
        input_config.get_mandatory_parsed_option::<f32>("SDF_RADIUS_MULTIPLIER", None)? / 100.0;

    let cmd_arg_sdf_divisions: f32 =
        input_config.get_mandatory_parsed_option("SDF_DIVISIONS", None)?;
    if !(9.9..600.1).contains(&cmd_arg_sdf_divisions) {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of SDF_DIVISIONS is [{}..{}[% :({})",
            10, 600, cmd_arg_sdf_divisions
        )));
    }

    // we already tested a_command.models.len()
    let input_model = &models[0];

    println!("Rust: model.vertices:{:?}, ", input_model.vertices.len());

    let aabb = parse_input(input_model)?;
    let (voxel_size, mesh) = build_voxel(
        cmd_arg_sdf_radius_multiplier,
        cmd_arg_sdf_divisions,
        input_model.vertices,
        input_model.indices,
        aabb,
        true,
    )?;
    let world_to_local = input_model.get_world_to_local_transform()?;
    if world_to_local.is_some() {
        println!(
            "Rust: applying world-local transformation 1/{:?}",
            input_model.world_orientation
        );
    } else {
        println!("Rust: *not* applying world-local transformation");
    };

    let output_model = build_output_model(voxel_size, mesh, world_to_local, true)?;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );
    if let Some(mv) = input_config.get_optional_parsed_option::<f32>(ffi::VERTEX_MERGE_TAG)? {
        // we take the easy way out here, and let blender do the de-duplication of the vertices.
        let _ = return_config.insert(ffi::VERTEX_MERGE_TAG.to_string(), mv.to_string());
    }
    println!(
        "Rust: SDF mesh operation returning {} vertices, {} indices",
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
