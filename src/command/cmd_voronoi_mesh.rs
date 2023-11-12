// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{
    command::{ConfigType, Model, Options, OwnedModel},
    ffi::FFIVector3,
    utils::{voronoi_utils::triangulate_face, GrowingVob, VertexDeduplicator3D},
    HallrError,
};
use boostvoronoi as BV;
use boostvoronoi::OutputType;
use centerline::{HasMatrix4, Matrix4};
use hronn::prelude::ConvertTo;
use itertools::Itertools;
use linestring::{
    linestring_2d::{Aabb2, VoronoiParabolicArc},
    linestring_3d::Plane,
};
use vector_traits::{
    approx::{AbsDiffEq, UlpsEq},
    num_traits::{AsPrimitive, Float},
    GenericScalar, GenericVector2, GenericVector3, HasXY,
};

mod impls;
#[cfg(test)]
mod tests;

/// Todo: clean this struct of any protobuf stuff
//#[derive(Default)]
struct DiagramHelperRw<T: GenericVector3> {
    /// a map between hash:able 2d coordinates and the known vertex index of pb_vertices
    vertex_map: VertexDeduplicator3D<T>,
}

type Face = Vec<usize>;

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

/// Helper structs that build PB buffer from Diagram
/// This construct contains the read-only items
struct DiagramHelperRo<T: GenericVector3 + HasMatrix4>
where
    T::Scalar: OutputType,
{
    diagram: BV::Diagram<T::Scalar>,
    vertices: Vec<BV::Point<i64>>,
    segments: Vec<BV::Line<i64>>,
    //aabb: Aabb2<f64>,
    // this list uses the diagram::Edge id as index
    rejected_edges: vob::Vob<u32>,
    // this list uses the diagram::Vertex id as index
    internal_vertices: vob::Vob<u32>,
    inverted_transform: T::Matrix4Type,
}

impl<T: GenericVector3> DiagramHelperRo<T>
where
    T: HasMatrix4 + ConvertTo<FFIVector3>,
    T::Scalar: OutputType,
    i64: AsPrimitive<T::Scalar>,
    f32: AsPrimitive<T::Scalar>,
{
    /// Retrieves a point from the voronoi input in the order it was presented to
    /// the voronoi builder
    #[inline(always)]
    fn retrieve_point(&self, cell_id: BV::CellIndex) -> Result<BV::Point<i64>, HallrError> {
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
    fn retrieve_segment(&self, cell_id: BV::CellIndex) -> Result<&BV::Line<i64>, HallrError> {
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
    fn convert_secondary_edge(&self, edge: &BV::Edge) -> Result<Vec<T>, HallrError> {
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
    fn convert_edge(
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
            self.retrieve_segment(twin_cell_id)?
        } else {
            self.retrieve_segment(cell_id)?
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
    fn convert_edges(
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
    fn split_pb_face_by_segment(
        &self,
        v0n: usize,
        v1n: usize,
        pb_face: &Vec<usize>,
    ) -> Result<Option<(Face, Face)>, HallrError> {
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
    fn iterate_cells(
        &self,
        mut dhrw: DiagramHelperRw<T>,
        edge_map: ahash::AHashMap<usize, Vec<usize>>,
    ) -> Result<(Face, Vec<T>), HallrError> {
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
}

#[allow(clippy::type_complexity)]
fn parse_input<T: GenericVector3 + HasMatrix4>(
    input_model: &Model<'_>,
    cmd_arg_max_voronoi_dimension: T::Scalar,
) -> Result<
    (
        Vec<BV::Point<i64>>,
        Vec<BV::Line<i64>>,
        Aabb2<T::Vector2>,
        T::Matrix4Type,
    ),
    HallrError,
>
where
    FFIVector3: ConvertTo<T>,
{
    let mut aabb = linestring::linestring_3d::Aabb3::<T>::default();
    for v in input_model.vertices.iter() {
        aabb.update_point(v.to())
    }

    let (plane, transform, vor_aabb)= centerline::get_transform_relaxed(
        aabb,
        cmd_arg_max_voronoi_dimension,
        T::Scalar::default_epsilon(),
        T::Scalar::default_max_ulps(),
    ).map_err(|_|{
        let aabb_d:T = aabb.get_high().unwrap() - aabb.get_low().unwrap();
        let aabb_c:T = (aabb.get_high().unwrap() + aabb.get_low().unwrap())/2.0.into();
        HallrError::InputNotPLane(format!(
            "Input data not in one plane and/or plane not intersecting origin: Δ({},{},{}) C({},{},{})",
            aabb_d.x(), aabb_d.y(), aabb_d.z(), aabb_c.x(), aabb_c.y(), aabb_c.z()))
    })?;

    if plane != Plane::XY {
        return Err(HallrError::InvalidInputData(format!("At the moment the voronoi mesh operation only supports input data in the XY plane. {:?}", plane)));
    }

    let inverse_transform = transform.safe_inverse().ok_or(HallrError::InternalError(
        "Could not calculate inverse matrix".to_string(),
    ))?;

    println!("voronoi: data was in plane:{:?} aabb:{:?}", plane, aabb);

    //println!("input Lines:{:?}", input_pb_model.vertices);

    let mut vor_lines = Vec::<BV::Line<i64>>::with_capacity(input_model.indices.len() / 2);
    let vor_vertices: Vec<BV::Point<i64>> = input_model
        .vertices
        .iter()
        .map(|vertex| {
            let p = transform
                .transform_point3(T::new_3d(vertex.x.into(), vertex.y.into(), vertex.z.into()))
                .to_2d();
            BV::Point {
                x: p.x().as_(),
                y: p.y().as_(),
            }
        })
        .collect();
    let mut used_vertices = vob::Vob::<u32>::fill(vor_vertices.len());

    for chunk in input_model.indices.chunks(2) {
        let v0 = chunk[0];
        let v1 = chunk[1];

        vor_lines.push(BV::Line {
            start: vor_vertices[v0],
            end: vor_vertices[v1],
        });
        let _ = used_vertices.set(v0, true);
        let _ = used_vertices.set(v1, true);
    }
    // save the unused vertices as points
    let vor_vertices: Vec<BV::Point<i64>> = vor_vertices
        .into_iter()
        .enumerate()
        .filter(|x| !used_vertices[x.0])
        .map(|x| x.1)
        .collect();
    Ok((vor_vertices, vor_lines, vor_aabb, inverse_transform))
}

/// from the list of rejected edges, find a list of internal (non-rejected) vertices.
fn find_internal_vertices<T: GenericVector3>(
    diagram: &BV::Diagram<T::Scalar>,
    rejected_edges: &vob::Vob<u32>,
) -> Result<vob::Vob<u32>, HallrError>
where
    T::Scalar: OutputType,
{
    let mut internal_vertices = vob::Vob::<u32>::fill(diagram.vertices().len());
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

/// Runs boost voronoi over the input and generates to output model.
/// Removes the external edges as we can't handle infinite length edges in blender.
fn compute_voronoi_mesh<T: GenericVector3>(
    input_pb_model: &Model<'_>,
    cmd_arg_max_voronoi_dimension: T::Scalar,
    cmd_discretization_distance: T::Scalar,
) -> Result<OwnedModel, HallrError>
where
    T: HasMatrix4,
    f32: AsPrimitive<T::Scalar>,
    i64: AsPrimitive<T::Scalar>,
    T::Scalar: OutputType,
    T: ConvertTo<FFIVector3>,
    FFIVector3: ConvertTo<T>,
{
    let (vor_vertices, vor_lines, vor_aabb2, inverted_transform) =
        parse_input::<T>(input_pb_model, cmd_arg_max_voronoi_dimension)?;
    let vor_diagram = {
        BV::Builder::<i64, T::Scalar>::default()
            .with_vertices(vor_vertices.iter())?
            .with_segments(vor_lines.iter())?
            .build()?
    };

    let discretization_distance: T::Scalar = {
        let max_dist: T::Vector2 = vor_aabb2.high().unwrap() - vor_aabb2.low().unwrap();
        cmd_discretization_distance * max_dist.magnitude() / 100.0.into()
    };

    let reject_edges = crate::utils::voronoi_utils::reject_external_edges::<T>(&vor_diagram)?;
    let internal_vertices = find_internal_vertices::<T>(&vor_diagram, &reject_edges)?;
    let diagram_helper = DiagramHelperRo::<T> {
        vertices: vor_vertices,
        segments: vor_lines,
        diagram: vor_diagram,
        rejected_edges: reject_edges,
        internal_vertices,
        inverted_transform,
    };

    let (dhrw, mod_edges) = diagram_helper.convert_edges(discretization_distance)?;
    let (indices, vertices) = diagram_helper.iterate_cells(dhrw, mod_edges)?;

    Ok(OwnedModel {
        //name: input_pb_model.name.clone(),
        //world_orientation: input_pb_model.world_orientation.clone(),
        indices,
        vertices: vertices.into_iter().map(|v| v.to()).collect(),
    })
}

/// Run the voronoi_mesh command
pub(crate) fn process_command<T: GenericVector3>(
    config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<(Vec<FFIVector3>, Face, ConfigType), HallrError>
where
    T: ConvertTo<FFIVector3> + HasMatrix4,
    FFIVector3: ConvertTo<T>,
    T::Scalar: OutputType,
    i64: AsPrimitive<T::Scalar>,
    T::Scalar: AsPrimitive<i64>,
    f32: AsPrimitive<T::Scalar>,
{
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

    let cmd_arg_max_voronoi_dimension: T::Scalar = config.get_mandatory_parsed_option(
        "MAX_VORONOI_DIMENSION",
        Some(super::DEFAULT_MAX_VORONOI_DIMENSION.as_()),
    )?;

    if !(super::DEFAULT_MAX_VORONOI_DIMENSION as i64..100_000_000)
        .contains(&cmd_arg_max_voronoi_dimension.as_())
    {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of MAX_VORONOI_DIMENSION is [{}..100_000_000[% :({})",
            super::DEFAULT_MAX_VORONOI_DIMENSION,
            cmd_arg_max_voronoi_dimension
        )));
    }
    let cmd_arg_discretization_distance: T::Scalar = config.get_mandatory_parsed_option(
        "DISTANCE",
        Some(super::DEFAULT_VORONOI_DISCRETE_DISTANCE.as_()),
    )?;

    if !(super::DEFAULT_VORONOI_DISCRETE_DISTANCE.as_()..5.0.into())
        .contains(&cmd_arg_discretization_distance)
    {
        return Err(HallrError::InvalidInputData(format!(
            "The valid range of DISTANCE is [{}..5.0[% :({})",
            super::DEFAULT_VORONOI_DISCRETE_DISTANCE,
            cmd_arg_discretization_distance
        )));
    }

    // used for simplification and discretization distance
    let max_distance =
        cmd_arg_max_voronoi_dimension * cmd_arg_discretization_distance / 100.0.into();
    // we already tested a_command.models.len()
    let input_model = &models[0];

    // we already tested that there is only one model

    //println!("model.name:{:?}, ", input_model.name);
    println!("model.vertices:{:?}, ", input_model.vertices.len());
    //println!("model.faces:{:?}, ", input_model.faces.len());
    //println!(
    //    "model.world_orientation:{:?}, ",
    //    input_model.world_orientation.as_ref().map_or(0, |_| 16)
    //);
    println!("MAX_VORONOI_DIMENSION:{:?}", cmd_arg_max_voronoi_dimension);
    println!(
        "VORONOI_DISCRETE_DISTANCE:{:?}%",
        cmd_arg_discretization_distance
    );
    println!("max_distance:{:?}", max_distance);
    println!();

    // do the actual operation
    let output_model = compute_voronoi_mesh(
        input_model,
        cmd_arg_max_voronoi_dimension,
        cmd_arg_discretization_distance,
    )?;

    let mut return_config = ConfigType::new();
    let _ = return_config.insert("mesh.format".to_string(), "triangulated".to_string());
    println!(
        "voronoi mesh operation returning {} vertices, {} indices",
        output_model.vertices.len(),
        output_model.indices.len()
    );
    Ok((output_model.vertices, output_model.indices, return_config))
}
