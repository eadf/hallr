// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use super::{GrowingVob, HallrError, VertexDeduplicator3D};
use crate::ffi::FFIVector3;
use boostvoronoi as BV;
use centerline::{HasMatrix4, Matrix4};
use hronn::prelude::ConvertTo;
use itertools::Itertools;
use linestring::linestring_2d::VoronoiParabolicArc;
use std::collections::VecDeque;
use vector_traits::{
    num_traits::{AsPrimitive, Float},
    GenericScalar, GenericVector2, GenericVector3, HasXY,
};

/// Mark infinite edges and their adjacent edges as EXTERNAL.
pub(crate) fn reject_external_edges<T: GenericVector3>(
    diagram: &BV::Diagram<T::Scalar>,
) -> Result<vob::Vob<u32>, HallrError>
where
    T::Scalar: BV::OutputType,
{
    let mut rejected_edges = vob::Vob::<u32>::fill_with_false(diagram.edges().len());

    for edge in diagram.edges().iter() {
        let edge = edge.get();
        let edge_id = edge.id();

        if diagram.edge_is_infinite(edge_id)? {
            mark_connected_edges::<T>(diagram, edge_id, &mut rejected_edges)?;
        }
    }
    Ok(rejected_edges)
}

/// Marks this edge and all other edges connecting to it via vertex1.
/// Repeat stops when connecting to input geometry.
/// if 'initial' is set to true it will search both ways, edge and the twin edge.
/// 'initial' will be set to false when going past the first edge
/// Note that this is not a recursive function (as it is in boostvoronoi)
pub(crate) fn mark_connected_edges<T: GenericVector3>(
    diagram: &BV::Diagram<T::Scalar>,
    edge_id: BV::EdgeIndex,
    marked_edges: &mut vob::Vob<u32>,
) -> Result<(), HallrError>
where
    T::Scalar: BV::OutputType,
{
    let mut initial = true;
    let mut queue = VecDeque::<BV::EdgeIndex>::new();
    queue.push_front(edge_id);

    'outer: while !queue.is_empty() {
        // unwrap is safe since we just checked !queue.is_empty()
        let edge_id = queue.pop_back().unwrap();

        if marked_edges.get_f(edge_id.0) {
            initial = false;
            continue 'outer;
        }

        let v1 = diagram.edge_get_vertex1(edge_id)?;
        if diagram.edge_get_vertex0(edge_id)?.is_some() && v1.is_none() {
            // this edge leads to nowhere
            let _ = marked_edges.set(edge_id.0, true);
            initial = false;
            continue 'outer;
        }
        let _ = marked_edges.set(edge_id.0, true);

        #[allow(unused_assignments)]
        if initial {
            initial = false;
            queue.push_back(diagram.edge_get_twin(edge_id)?);
        } else {
            let _ = marked_edges.set(diagram.edge_get_twin(edge_id)?.0, true);
        }

        if v1.is_none()
            || !diagram.edges()[(Some(edge_id))
                .ok_or_else(|| HallrError::InternalError("Could not get edge twin".to_string()))?
                .0]
                .get()
                .is_primary()
        {
            // stop traversing this line if vertex1 is not found or if the edge is not primary
            initial = false;
            continue 'outer;
        }
        // v1 is always Some from this point on
        if let Some(v1) = v1 {
            let v1 = diagram.vertex_get(v1)?.get();
            if v1.is_site_point() {
                // stop iterating line when site points detected
                initial = false;
                continue 'outer;
            }
            //self.reject_vertex(v1, color);
            let mut edge_iter = v1.get_incident_edge()?;
            let v_incident_edge = edge_iter;
            loop {
                if !marked_edges.get_f(edge_iter.0) {
                    queue.push_back(edge_iter);
                }
                edge_iter = diagram.edge_rot_next(edge_iter)?;
                if edge_iter == v_incident_edge {
                    break;
                }
            }
        }
        initial = false;
    }
    Ok(())
}

const DUMMY_VEC: [usize; 0] = [];

/// Triangulates a Voronoi site, also known as a face, and inserts the resulting triangles as indices
/// into the provided `indices` vector.
/// This will triangulate a face that is in principle defined in the XY plane, or close to.
/// The general use case is to triangulate cmd_voronoi sites that are defined in the XY plane.
///
/// # Arguments
///
/// * `indices`: A mutable reference to a vector where the triangulation indices will be inserted.
/// * `vertices`: A slice of vertices that represent the original points.
/// * `face`: A slice of indices referencing vertices that define the Voronoi face.
///
/// # Errors
///
/// Returns a `Result<(), HallrError>` where `HallrError` represents any potential error during the
/// triangulation process.
///
/// # Type Parameters
///
/// * `T`: A type that implements the `GenericVector3` trait.
///
pub fn triangulate_face<T: GenericVector3>(
    indices: &mut Vec<usize>,
    vertices: &[T],
    face: &[usize],
) -> Result<(), HallrError>
where
    T::Scalar: Float,
{
    match face.len() {
        0..=2 => Err(HallrError::InternalError(format!(
            "Detected a cmd_voronoi face with too few indices:{}",
            face.len()
        )))?,
        3 => indices.extend(face.iter()),
        _ => {
            let mut flattened_coords = Vec::<T::Scalar>::with_capacity(face.len() * 2);
            for i in face {
                let v = vertices[*i];
                flattened_coords.push(v.x());
                flattened_coords.push(v.y());
            }

            let triangulation = earcutr::earcut(&flattened_coords, &DUMMY_VEC, 2)?;
            for i in triangulation {
                indices.push(face[i]);
            }
        }
    }
    Ok(())
}

//#[derive(Default)]
pub(crate) struct DiagramHelperRw<T: GenericVector3> {
    /// a map between hash:able 2d coordinates and the known vertex index of pb_vertices
    vertex_map: VertexDeduplicator3D<T>,
}

impl<T: GenericVector3> DiagramHelperRw<T>
where
    T: ConvertTo<FFIVector3>,
{
    /// transform the voronoi Point into a PB point. Perform duplication checks
    #[inline(always)]
    fn place_new_vertex_dup_check(&mut self, vertex: T) -> Result<usize, HallrError> {
        let rv = self.vertex_map.get_index_or_insert(vertex)? as usize;
        Ok(rv)
    }

    /// Place the point in the list. Does not perform any de-duplication checks
    #[allow(dead_code)]
    #[inline(always)]
    fn place_new_vertex_unchecked(&mut self, vertex: T) -> Result<usize, HallrError> {
        let n = self.vertex_map.vertices.len();
        self.vertex_map.vertices.push(vertex);
        Ok(n)
    }
}

/// Helper structs that build vertices and indices from a voronoi diagram
/// This construct contains the read-only items
pub(crate) struct DiagramHelperRo<T: GenericVector3 + HasMatrix4>
where
    T::Scalar: BV::OutputType,
{
    pub(crate) diagram: BV::Diagram<T::Scalar>,
    pub(crate) vertices: Vec<BV::Point<i64>>,
    pub(crate) segments: Vec<BV::Line<i64>>,
    //aabb: Aabb2<f64>,
    // this list uses the diagram::Edge id as index
    pub(crate) rejected_edges: vob::Vob<u32>,
    // this list uses the diagram::Vertex id as index
    pub(crate) internal_vertices: vob::Vob<u32>,
    pub(crate) inverted_transform: T::Matrix4Type,
}

impl<T: GenericVector3> DiagramHelperRo<T>
where
    T: HasMatrix4 + ConvertTo<FFIVector3>,
    T::Scalar: BV::OutputType,
    i64: AsPrimitive<T::Scalar>,
    f32: AsPrimitive<T::Scalar>,
{
    /// Retrieves a point from the voronoi input in the order it was presented to
    /// the voronoi builder
    #[inline(always)]
    pub(crate) fn retrieve_point(
        &self,
        cell_id: BV::CellIndex,
    ) -> Result<BV::Point<i64>, HallrError> {
        let (index, cat) = self.diagram.get_cell(cell_id)?.get().source_index_2();
        Ok(match cat {
            BV::SourceCategory::SinglePoint => self.vertices[index],
            BV::SourceCategory::SegmentStart => self.segments[index - self.vertices.len()].start,
            BV::SourceCategory::Segment | BV::SourceCategory::SegmentEnd => {
                self.segments[index - self.vertices.len()].end
            }
        })
    }

    /// Retrieves a segment from the voronoi input in the order it was presented to
    /// the voronoi builder
    #[inline(always)]
    pub(crate) fn retrieve_segment(
        &self,
        cell_id: BV::CellIndex,
    ) -> Result<&BV::Line<i64>, HallrError> {
        let cell = self.diagram.get_cell(cell_id)?.get();
        let index = cell.source_index() - self.vertices.len();
        Ok(&self.segments[index])
    }

    /// Convert a secondary edge into a set of vertices.
    /// [maybe start, one mid, maybe end point]
    /// Secondary edges are a bit tricky because they lack the mid-point vertex where they
    /// intersect with the segment that created the edge. So we need to re-create it.
    /// Secondary edges can also be half internal and half external i.e. the two vertices may
    /// be on opposite sides of the inside/outside boundary.
    pub(crate) fn convert_secondary_edge(&self, edge: &BV::Edge) -> Result<Vec<T>, HallrError> {
        let edge_id = edge.id();
        let edge_twin_id = self.diagram.edge_get_twin(edge_id)?;
        let cell_id = self.diagram.edge_get_cell(edge_id)?;
        let cell = self.diagram.get_cell(cell_id)?.get();
        let twin_cell_id = self.diagram.get_edge(edge_twin_id)?.get().cell().unwrap();

        let segment = if cell.contains_point() {
            self.retrieve_segment(twin_cell_id)?
        } else {
            self.retrieve_segment(cell_id)?
        };
        let vertex0_id = edge.vertex0();
        let vertex1_id = self.diagram.edge_get_vertex1(edge_id)?;

        let start_point = if let Some(vertex0_id) = vertex0_id {
            let vertex0 = self.diagram.vertex_get(vertex0_id)?.get();
            if !self.internal_vertices[vertex0.get_id().0] {
                None
            } else if vertex0.is_site_point() {
                Some(T::new_3d(vertex0.x(), vertex0.y(), T::Scalar::ZERO))
            } else {
                Some(T::new_3d(vertex0.x(), vertex0.y(), f32::NAN.as_()))
            }
        } else {
            None
        };

        let end_point = if let Some(vertex1_id) = vertex1_id {
            let vertex1 = self.diagram.vertex_get(vertex1_id)?.get();
            if !self.internal_vertices[vertex1.get_id().0] {
                None
            } else if vertex1.is_site_point() {
                Some(T::new_3d(vertex1.x(), vertex1.y(), T::Scalar::ZERO))
            } else {
                Some(T::new_3d(vertex1.x(), vertex1.y(), f32::NAN.as_()))
            }
        } else {
            None
        };

        let cell_point = {
            let cell_point = if cell.contains_point() {
                self.retrieve_point(cell_id)?
            } else {
                self.retrieve_point(twin_cell_id)?
            };
            T::Vector2::new_2d(cell_point.x.as_(), cell_point.y.as_())
        };

        let segment = linestring::linestring_2d::Line2::<T::Vector2>::from([
            segment.start.x.as_(),
            segment.start.y.as_(),
            segment.end.x.as_(),
            segment.end.y.as_(),
        ]);

        let mut samples = Vec::<T>::new();

        if let Some(mut start_point) = start_point {
            if start_point.z().is_finite() {
                samples.push(start_point);
            } else {
                start_point.set_z(if cell.contains_point() {
                    -cell_point.distance(start_point.to_2d())
                } else {
                    -linestring::linestring_2d::distance_to_line_squared_safe(
                        segment.start,
                        segment.end,
                        start_point.to_2d(),
                    )
                    .sqrt()
                });
                samples.push(start_point);
            }
        }

        samples.push(T::new_3d(cell_point.x(), cell_point.y(), T::Scalar::ZERO));

        if let Some(mut end_point) = end_point {
            if end_point.z().is_finite() {
                samples.push(end_point);
            } else {
                end_point.set_z(if cell.contains_point() {
                    -cell_point.distance(end_point.to_2d())
                } else {
                    -linestring::linestring_2d::distance_to_line_squared_safe(
                        segment.start,
                        segment.end,
                        end_point.to_2d(),
                    )
                    .sqrt()
                });
                samples.push(end_point);
            }
        }
        Ok(samples)
    }

    /// Convert an edge into a set of vertices
    /// primary edges: [start, end point]
    /// curved edges, [start, multiple mid, end point]
    /// todo: try to consolidate code with convert_secondary_edge()
    pub(crate) fn convert_edge(
        &self,
        edge: &BV::Edge,
        discretization_distance: T::Scalar,
    ) -> Result<Vec<T>, HallrError> {
        let edge_id = edge.id();
        let edge_twin_id = self.diagram.edge_get_twin(edge_id)?;
        let cell_id = self.diagram.edge_get_cell(edge_id)?;
        let cell = self.diagram.get_cell(cell_id)?.get();
        let twin_cell_id = self.diagram.get_edge(edge_twin_id)?.get().cell()?;
        let segment = if cell.contains_point() {
            let twin_cell = self.diagram.get_cell(twin_cell_id)?.get();
            if twin_cell.contains_point() {
                let cell_point = self.retrieve_point(cell_id)?;
                BV::Line::new(cell_point, cell_point)
            } else {
                *self.retrieve_segment(twin_cell_id)?
            }
        } else {
            *self.retrieve_segment(cell_id)?
        };

        let (start_point, startpoint_is_internal) = if let Some(vertex0) = edge.vertex0() {
            let vertex0 = self.diagram.vertex_get(vertex0)?.get();

            let start_point = if vertex0.is_site_point() {
                T::new_3d(vertex0.x(), vertex0.y(), T::Scalar::ZERO)
            } else {
                T::new_3d(vertex0.x(), vertex0.y(), f32::NAN.as_())
            };
            (start_point, self.internal_vertices[vertex0.get_id().0])
        } else {
            return Err(HallrError::InternalError(format!(
                "Edge vertex0 could not be found. {}:{}",
                file!(),
                line!()
            )));
        };

        let (end_point, end_point_is_internal) =
            if let Some(vertex1) = self.diagram.edge_get_vertex1(edge_id)? {
                let vertex1 = self.diagram.vertex_get(vertex1)?.get();

                let end_point = if vertex1.is_site_point() {
                    T::new_3d(vertex1.x(), vertex1.y(), T::Scalar::ZERO)
                } else {
                    T::new_3d(vertex1.x(), vertex1.y(), f32::NAN.as_())
                };
                (end_point, self.internal_vertices[vertex1.get_id().0])
            } else {
                return Err(HallrError::InternalError(format!(
                    "Edge vertex1 could not be found. {}:{}",
                    file!(),
                    line!()
                )));
            };

        let cell_point = if cell.contains_point() {
            self.retrieve_point(cell_id)?
        } else {
            self.retrieve_point(twin_cell_id)?
        };
        let cell_point = T::Vector2::new_2d(cell_point.x.as_(), cell_point.y.as_());

        let segment = linestring::linestring_2d::Line2::<T::Vector2>::from([
            segment.start.x.as_(),
            segment.start.y.as_(),
            segment.end.x.as_(),
            segment.end.y.as_(),
        ]);

        let mut samples = Vec::<T>::new();

        if edge.is_curved() {
            let arc = VoronoiParabolicArc::new(
                segment,
                cell_point,
                start_point.to_2d(),
                end_point.to_2d(),
            );

            for p in arc.discretize_3d(discretization_distance).iter() {
                samples.push(*p);
            }
        } else {
            if startpoint_is_internal {
                if start_point.z().is_finite() {
                    samples.push(start_point);
                } else {
                    let z_comp = if cell.contains_point() {
                        -cell_point.distance(start_point.to_2d())
                    } else {
                        -linestring::linestring_2d::distance_to_line_squared_safe(
                            segment.start,
                            segment.end,
                            start_point.to_2d(),
                        )
                        .sqrt()
                    };
                    samples.push(T::new_3d(start_point.x(), start_point.y(), z_comp));
                }
            }

            if end_point_is_internal {
                if end_point.z().is_finite() {
                    samples.push(end_point);
                } else {
                    let z_comp = if cell.contains_point() {
                        -cell_point.distance(end_point.to_2d())
                    } else {
                        -linestring::linestring_2d::distance_to_line_squared_safe(
                            segment.start,
                            segment.end,
                            end_point.to_2d(),
                        )
                        .sqrt()
                    };
                    samples.push(T::new_3d(end_point.x(), end_point.y(), z_comp));
                }
            }
        }

        Ok(samples)
    }

    /// convert the edges of the diagram into a list of vertices
    #[allow(clippy::type_complexity)]
    pub(crate) fn convert_edges(
        &self,
        discretization_distance: T::Scalar,
    ) -> Result<(DiagramHelperRw<T>, ahash::AHashMap<usize, Vec<usize>>), HallrError> {
        let mut hrw = DiagramHelperRw::default();
        let mut rv = ahash::AHashMap::<usize, Vec<usize>>::new();

        for edge in self.diagram.edges() {
            let edge = edge.get();
            let edge_id = edge.id();
            // secondary edges may be in the rejected list while still contain needed data
            if !edge.is_secondary() && self.rejected_edges[edge_id.0] {
                // ignore rejected edges, but only non-secondary ones.
                continue;
            }

            let twin_id = edge.twin()?;

            //println!("edge:{:?}", edge_id.0);
            if !rv.contains_key(&twin_id.0) {
                let samples = if edge.is_secondary() {
                    self.convert_secondary_edge(&edge)?
                } else {
                    self.convert_edge(&edge, discretization_distance)?
                };
                let mut pb_edge: Vec<usize> = Vec::with_capacity(samples.len());
                for coord in samples {
                    let v = hrw.place_new_vertex_dup_check(coord)?;
                    if !pb_edge.contains(&v) {
                        pb_edge.push(v);
                    }
                }

                let _ = rv.insert(edge_id.0, pb_edge);
            } else {
                // ignore edge because the twin is already processed
            }
        }
        Ok((hrw, rv))
    }

    /// if a cell contains a segment the pb_face should be split into two faces, one
    /// on each side of the segment.
    #[allow(clippy::type_complexity)]
    fn split_pb_face_by_segment(
        &self,
        v0n: usize,
        v1n: usize,
        pb_face: &[usize],
    ) -> Result<Option<(Vec<usize>, Vec<usize>)>, HallrError> {
        if let Some(v0i) = pb_face.iter().position(|x| x == &v0n) {
            if let Some(v1i) = pb_face.iter().position(|x| x == &v1n) {
                let mut a = Vec::<usize>::new();
                let b: Vec<usize> = if v0i < v1i {
                    // todo, could this be made into a direct .collect() too?
                    for i in (0..=v0i).chain(v1i..pb_face.len()) {
                        a.push(pb_face[i]);
                    }
                    pb_face.iter().take(v1i + 1).skip(v0i).cloned().collect()
                } else {
                    // todo, could this be made into a direct .collect() too?
                    for i in (0..=v1i).chain(v0i..pb_face.len()) {
                        a.push(pb_face[i]);
                    }
                    pb_face.iter().take(v0i + 1).skip(v1i).cloned().collect()
                };
                return Ok(Some((a, b)));
            }
        }
        Ok(None)
    }

    /// Iterate over each cell, generate mesh
    pub(crate) fn generate_mesh_from_cells(
        &self,
        mut dhrw: DiagramHelperRw<T>,
        edge_map: ahash::AHashMap<usize, Vec<usize>>,
    ) -> Result<(Vec<usize>, Vec<T>), HallrError> {
        let mut return_indices = Vec::<usize>::new();

        for cell in self.diagram.cells().iter() {
            let cell = cell.get();
            let cell_id = cell.id();

            if cell.contains_point() {
                let cell_point = {
                    let cp = self.retrieve_point(cell_id)?;
                    dhrw.place_new_vertex_dup_check(T::new_3d(
                        cp.x.as_(),
                        cp.y.as_(),
                        T::Scalar::ZERO,
                    ))?
                };

                for edge_id in self.diagram.cell_edge_iterator(cell_id) {
                    let edge = self.diagram.get_edge(edge_id)?.get();
                    let twin_id = edge.twin()?;

                    if self.rejected_edges[edge_id.0] && !edge.is_secondary() {
                        continue;
                    }
                    let mod_edge: Box<dyn ExactSizeIterator<Item = &usize>> = {
                        if let Some(e) = edge_map.get(&edge_id.0) {
                            Box::new(e.iter())
                        } else {
                            Box::new(
                                edge_map
                                    .get(&twin_id.0)
                                    .ok_or_else(|| {
                                        HallrError::InternalError(format!(
                                            "could not get twin edge, {}, {}",
                                            file!(),
                                            line!()
                                        ))
                                    })?
                                    .iter()
                                    .rev(),
                            )
                        }
                    };

                    for (a, b) in mod_edge.tuple_windows::<(_, _)>() {
                        let a = *a;
                        let b = *b;

                        if a != cell_point && b != cell_point {
                            let mut pb_face = Vec::new();
                            let mut face = vec![a, b, cell_point];
                            pb_face.append(&mut face);
                            //print!(" pb:{:?},", pb_face.vertices);
                            if pb_face.len() > 2 {
                                triangulate_face(
                                    &mut return_indices,
                                    &dhrw.vertex_map.vertices,
                                    &pb_face,
                                )?
                            } else {
                                //print!("ignored ");
                            }
                        }
                    }
                }
                //println!();
            }
            if cell.contains_segment() {
                let segment = self.retrieve_segment(cell_id)?;
                let v0n = dhrw.place_new_vertex_dup_check(T::new_3d(
                    segment.start.x.as_(),
                    segment.start.y.as_(),
                    T::Scalar::ZERO,
                ))?;
                let v1n = dhrw.place_new_vertex_dup_check(T::new_3d(
                    segment.end.x.as_(),
                    segment.end.y.as_(),
                    T::Scalar::ZERO,
                ))?;
                //print!("SCell:{} v0:{} v1:{} ", cell_id.0, v0n, v1n);
                let mut new_face = Vec::new();
                for edge_id in self.diagram.cell_edge_iterator(cell_id) {
                    let edge = self.diagram.get_edge(edge_id)?.get();
                    let twin_id = edge.twin()?;

                    let mod_edge: Box<dyn ExactSizeIterator<Item = &usize>> = {
                        if let Some(e) = edge_map.get(&edge_id.0) {
                            Box::new(e.iter())
                        } else if let Some(e) = edge_map.get(&twin_id.0) {
                            Box::new(e.iter().rev())
                        } else {
                            //let e:Option<usize> = None;
                            Box::new(None.iter())
                        }
                    };

                    for v in mod_edge {
                        //print! {"{:?},", v};
                        if !new_face.contains(v) {
                            new_face.push(*v);
                        }
                    }
                }

                if let Some((split_a, split_b)) =
                    self.split_pb_face_by_segment(v0n, v1n, &new_face)?
                {
                    if split_a.len() > 2 {
                        triangulate_face(&mut return_indices, &dhrw.vertex_map.vertices, &split_a)?;
                    }
                    if split_b.len() > 2 {
                        triangulate_face(&mut return_indices, &dhrw.vertex_map.vertices, &split_b)?;
                    }
                } else if new_face.len() > 2 {
                    triangulate_face(&mut return_indices, &dhrw.vertex_map.vertices, &new_face)?;
                }
            }
        }
        //println!("indices:{:?}", return_indices);
        //println!("vertices:{:?}", dhrw.vertex_map.vertices);
        let vertices = dhrw
            .vertex_map
            .vertices
            .into_iter()
            .map(|v| self.inverted_transform.transform_point3(v))
            .collect();
        Ok((return_indices, vertices))
    }

    /// Iterate over each cell, generate edges in "chunk" format
    pub(crate) fn generate_voronoi_edges_from_cells(
        &self,
        mut dhrw: DiagramHelperRw<T>,
        edge_map: ahash::AHashMap<usize, Vec<usize>>,
        cmd_arg_keep_input: bool,
    ) -> Result<(Vec<usize>, Vec<T>), HallrError> {
        // A vec containing the edges of the faces in "chunk" format
        let mut return_indices = Vec::<usize>::with_capacity(edge_map.len() * 3);
        for (_, edge) in edge_map.iter() {
            for line in edge.windows(2) {
                return_indices.extend(line);
            }
        }

        // lookup already existing vertex indices. TODO: figure out the indices from input
        if cmd_arg_keep_input {
            for line in self.segments.iter() {
                return_indices.push(dhrw.place_new_vertex_dup_check(T::new_3d(
                    line.start.x.as_(),
                    line.start.y.as_(),
                    T::Scalar::ZERO,
                ))?);
                return_indices.push(dhrw.place_new_vertex_dup_check(T::new_3d(
                    line.end.x.as_(),
                    line.end.y.as_(),
                    T::Scalar::ZERO,
                ))?);
            }
        }

        let vertices = dhrw
            .vertex_map
            .vertices
            .into_iter()
            .map(|v| self.inverted_transform.transform_point3(v))
            .collect();
        Ok((return_indices, vertices))
    }
}

impl<T: GenericVector3> Default for DiagramHelperRw<T> {
    fn default() -> Self {
        Self {
            vertex_map: VertexDeduplicator3D::<T>::default(),
        }
    }
}

/// from the list of rejected edges, find a list of internal (non-rejected) vertices.
pub(crate) fn find_internal_vertices<T: GenericVector3>(
    diagram: &BV::Diagram<T::Scalar>,
    rejected_edges: &vob::Vob<u32>,
) -> Result<vob::Vob<u32>, HallrError>
where
    T::Scalar: BV::OutputType,
{
    let mut internal_vertices = vob::Vob::<u32>::fill_with_false(diagram.vertices().len());
    for (_, e) in diagram
        .edges()
        .iter()
        .enumerate()
        .filter(|(eid, _)| !rejected_edges[*eid])
    {
        let e = e.get();
        if e.is_primary() {
            if let Some(v0) = e.vertex0() {
                let _ = internal_vertices.set(v0.0, true);
            }
            if let Some(v1) = diagram.edge_get_vertex1(e.id())? {
                let _ = internal_vertices.set(v1.0, true);
            }
        }
    }
    for (vid, _) in diagram
        .vertices()
        .iter()
        .enumerate()
        .filter(|(_, v)| v.get().is_site_point())
    {
        let _ = internal_vertices.set(vid, true);
    }
    Ok(internal_vertices)
}
