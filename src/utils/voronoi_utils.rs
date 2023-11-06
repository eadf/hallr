use super::{GrowingVob, HallrError};
use boostvoronoi as BV;
use boostvoronoi::OutputType;
use std::collections::VecDeque;
use vector_traits::GenericVector3;

/// Mark infinite edges and their adjacent edges as EXTERNAL.
pub(crate) fn reject_external_edges<T: GenericVector3>(
    diagram: &BV::Diagram<T::Scalar>,
) -> Result<vob::Vob<u32>, HallrError>
where
    T::Scalar: OutputType,
{
    let mut rejected_edges = vob::Vob::<u32>::fill(diagram.edges().len());

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
    T::Scalar: OutputType,
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
/// The general use case is to triangulate voronoi sites that are defined in the XY plane.
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
    T::Scalar: vector_traits::num_traits::Float,
{
    match face.len() {
        0..=2 => Err(HallrError::InternalError(format!(
            "Detected a voronoi face with too few indices:{}",
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
