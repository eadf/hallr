// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

#[cfg(test)]
mod tests;

use crate::{
    command::{ConfigType, Model, Options, OwnedModel},
    ffi::FFIVector3,
    HallrError,
};
use fast_surface_nets::{ndshape::ConstShape, surface_nets, SurfaceNetsBuffer};
use ilattice::{glam as iglam, prelude::Extent};
use rayon::prelude::*;
use std::time;

// The un-padded chunk side, it will become 16*16*16
const UN_PADDED_CHUNK_SIDE: u32 = 14_u32;
type PaddedChunkShape = fast_surface_nets::ndshape::ConstShape3u32<
    { UN_PADDED_CHUNK_SIDE + 2 },
    { UN_PADDED_CHUNK_SIDE + 2 },
    { UN_PADDED_CHUNK_SIDE + 2 },
>;
const DEFAULT_SDF_VALUE: f32 = 999.0;
type Extent3i = Extent<iglam::IVec3>;

/// returns a list of type-converted vertices, a list of edges, and a AABB (not padded by radius)
#[allow(clippy::type_complexity)]
fn parse_input(
    model: &Model<'_>,
) -> Result<(Vec<iglam::Vec3A>, Vec<(u32, u32)>, Extent<iglam::Vec3A>), HallrError> {
    let mut edges = Vec::<(u32, u32)>::default();
    let mut aabb: Option<Extent<iglam::Vec3A>> = None;

    let vertices: Result<Vec<_>, HallrError> = model
        .vertices
        .iter()
        .map(|vertex| {
            if !vertex.x.is_finite() || !vertex.y.is_finite() || !vertex.z.is_finite() {
                Err(HallrError::InvalidInputData(format!(
                    "Only finite coordinates are allowed ({},{},{})",
                    vertex.x, vertex.y, vertex.z
                )))?
            } else {
                let point = iglam::vec3a(vertex.x, vertex.y, vertex.z);
                let v_aabb = Extent::from_min_and_shape(point, iglam::Vec3A::splat(0.0));
                aabb = if let Some(aabb) = aabb {
                    Some(aabb.bound_union(&v_aabb))
                } else {
                    Some(v_aabb)
                };

                Ok(point)
            }
        })
        .collect();
    let vertices = vertices?;

    for chunk in model.indices.chunks_exact(2) {
        edges.push((chunk[0] as u32, chunk[1] as u32));
    }
    println!("edges.len():{}", edges.len());
    println!("aabb :{:?}", aabb);

    Ok((vertices, edges, aabb.unwrap()))
}

/// Build the chunk lattice and spawn off thread tasks for each chunk
fn build_voxel(
    radius_multiplier: f32,
    divisions: f32,
    vertices: &[iglam::Vec3A],
    edges: Vec<(u32, u32)>,
    unpadded_aabb: Extent<iglam::Vec3A>,
    verbose: bool,
) -> Result<
    (
        f32, // voxel_size
        Vec<(iglam::Vec3A /* offset */, SurfaceNetsBuffer)>,
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
            "Voxelizing using tube radius. {} = {}*{}*{}",
            radius, max_dimension, radius_multiplier, scale
        );

        println!(
            "Voxelizing using divisions = {}, max dimension = {}, scale factor={} (max_dimension*scale={})",
            divisions,
            max_dimension,
            scale,
            max_dimension * scale
        );
        println!();
    }
    let vertices: Vec<iglam::Vec3A> = vertices
        .iter()
        .map(|v| iglam::Vec3A::new(v.x, v.y, v.z) * scale)
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
        let unpadded_chunk_shape = iglam::IVec3::from([UN_PADDED_CHUNK_SIDE as i32; 3]);
        // Spawn off thread tasks creating and processing chunks.
        chunks_extent
            .iter3()
            .par_bridge()
            .filter_map(move |p| {
                let unpadded_chunk_extent =
                    Extent3i::from_min_and_shape(p * unpadded_chunk_shape, unpadded_chunk_shape);

                generate_and_process_sdf_chunk(unpadded_chunk_extent, &vertices, &edges, radius)
            })
            .collect()
    };

    if verbose {
        println!(
            "process_chunks() duration: {:?} generated {} chunks",
            now.elapsed(),
            sdf_chunks.len()
        );
    }

    Ok((1.0 / scale, sdf_chunks))
}

/// Generate the data of a single chunk
fn generate_and_process_sdf_chunk(
    unpadded_chunk_extent: Extent3i,
    vertices: &[iglam::Vec3A],
    edges: &[(u32, u32)],
    thickness: f32,
) -> Option<(iglam::Vec3A, SurfaceNetsBuffer)> {
    // the origin of this chunk, in voxel scale
    let padded_chunk_extent = unpadded_chunk_extent.padded(1);

    // filter out the edges that does not affect this chunk
    let filtered_edges: Vec<_> = edges
        .iter()
        .filter_map(|(e0, e1)| {
            let (e0, e1) = (*e0 as usize, *e1 as usize);
            let tube_extent = Extent::from_min_and_lub(
                vertices[e0].min(vertices[e1]) - iglam::Vec3A::from([thickness; 3]),
                vertices[e0].max(vertices[e1]) + iglam::Vec3A::from([thickness; 3]),
            )
            .containing_integer_extent();
            if !padded_chunk_extent.intersection(&tube_extent).is_empty() {
                // The AABB of the edge tube intersected this chunk - keep it
                Some((e0, e1))
            } else {
                None
            }
        })
        .collect();

    #[cfg(not(feature = "display_chunks"))]
    if filtered_edges.is_empty() {
        // no tubes intersected this chunk
        return None;
    }

    let mut array = { [DEFAULT_SDF_VALUE; PaddedChunkShape::SIZE as usize] };

    #[cfg(feature = "display_chunks")]
    // The corners of the un-padded chunk extent
    let corners: Vec<_> = unpadded_chunk_extent
        .corners3()
        .iter()
        .map(|p| p.to_float())
        .collect();

    let mut some_neg_or_zero_found = false;
    let mut some_pos_found = false;

    for pwo in padded_chunk_extent.iter3() {
        let v = {
            let p = pwo - unpadded_chunk_extent.minimum + 1;
            &mut array[PaddedChunkShape::linearize([p.x as u32, p.y as u32, p.z as u32]) as usize]
        };
        let pwo = pwo.as_vec3a();
        // Point With Offset from the un-padded extent minimum
        #[cfg(feature = "display_chunks")]
        {
            // todo: this could probably be optimized with PaddedChunkShape::linearize(corner_pos)
            let mut x = *v;
            for c in corners.iter() {
                x = x.min(c.distance(pwo) - 1.);
            }
            *v = (*v).min(x);
        }
        for (from_v, to_v) in filtered_edges
            .iter()
            .map(|(e0, e1)| (vertices[*e0], vertices[*e1]))
        {
            // This is the sdf formula of a capsule
            let pa = pwo - from_v;
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

/// Build the return model
pub(crate) fn build_output_model(
    //pb_model_name: String,
    //pb_world: Option<PB_Matrix4x432>,
    voxel_size: f32,
    mesh_buffers: Vec<(iglam::Vec3A, SurfaceNetsBuffer)>,
    verbose: bool,
) -> Result<OwnedModel, HallrError> {
    let now = time::Instant::now();

    let (mut vertices, mut indices) = {
        // calculate the maximum required vertices & facec capacity
        let (vertex_capacity, face_capacity) = mesh_buffers
            .iter()
            .fold((0_usize, 0_usize), |(v, f), chunk| {
                (v + chunk.1.positions.len(), f + chunk.1.indices.len())
            });
        if vertex_capacity >= u32::MAX as usize {
            return Err(HallrError::Overflow(
                format!("Generated mesh contains too many vertices to be referenced by u32: {}. Reduce the resolution.", vertex_capacity)));
        }

        if face_capacity >= u32::MAX as usize {
            return Err(HallrError::Overflow(
                format!("Generated mesh contains too many faces to be referenced by u32: {}. Reduce the resolution.", vertex_capacity)));
        }
        (
            Vec::with_capacity(vertex_capacity),
            Vec::with_capacity(face_capacity),
        )
    };

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
        //name: pb_model_name,
        vertices,
        indices,
    })
}

/// Run the voronoi_mesh command
pub(crate) fn process_command(
    config: ConfigType,
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

    let cmd_arg_sdf_radius_multiplier =
        config.get_mandatory_parsed_option::<f32>("SDF_RADIUS_MULTIPLIER", None)? / 100.0;

    let cmd_arg_sdf_divisions: f32 = config.get_mandatory_parsed_option("SDF_DIVISIONS", None)?;
    if !(9.9..600.1).contains(&cmd_arg_sdf_divisions) {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of SDF_DIVISIONS is [{}..{}[% :({})",
            10, 600, cmd_arg_sdf_divisions
        )));
    }

    // we already tested a_command.models.len()
    let input_model = &models[0];

    println!("model.vertices:{:?}, ", input_model.vertices.len());

    let (vertices, edges, aabb) = parse_input(input_model)?;
    let (voxel_size, mesh) = build_voxel(
        cmd_arg_sdf_radius_multiplier,
        cmd_arg_sdf_divisions,
        &vertices,
        edges,
        aabb,
        true,
    )?;

    let output_model = build_output_model(voxel_size, mesh, true)?;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert("mesh.format".to_string(), "triangulated".to_string());
    let _ = return_config.insert("REMOVE_DOUBLES".to_string(), "true".to_string());
    println!(
        "SDF mesh operation returning {} vertices, {} indices",
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
