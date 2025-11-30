// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

pub(crate) mod rounded_cones_fsn;
#[cfg(test)]
mod tests;
mod trait_impl;
pub(crate) mod voronoi_utils;

use crate::HallrError;
use hronn::prelude::MaximumTracker;
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;
use std::{cmp::Reverse, time::Instant};
use vector_traits::prelude::{GenericScalar, GenericVector2, GenericVector3, HasXYZ};

pub(crate) trait GrowingVob {
    fn fill_with_false(initial_size: usize) -> vob::Vob<u32>;
    #[allow(dead_code)]
    fn set_grow(&mut self, bit: usize, state: bool) -> bool;
    /// get with default value: false
    fn get_f(&self, bit: usize) -> bool;
}

impl GrowingVob for vob::Vob<u32> {
    fn fill_with_false(initial_size: usize) -> Self {
        vob::Vob::<u32>::from_elem_with_storage_type(false, initial_size)
    }

    #[inline]
    fn set_grow(&mut self, bit: usize, state: bool) -> bool {
        if bit >= self.len() {
            self.resize(bit + 512, false);
        }
        self.set(bit, state)
    }

    #[inline]
    fn get_f(&self, bit: usize) -> bool {
        self.get(bit).unwrap_or(false)
    }
}

#[allow(clippy::type_complexity)]
#[derive(Default)]
pub(crate) struct VertexDeduplicator2D<T: GenericVector2> {
    set: FxHashMap<
        (
            <T::Scalar as GenericScalar>::BitsType,
            <T::Scalar as GenericScalar>::BitsType,
        ),
        u32,
    >,
    pub vertices: Vec<T>,
}

impl<T: GenericVector2> VertexDeduplicator2D<T> {
    #[allow(dead_code)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            set: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            vertices: Vec::with_capacity(capacity),
        }
    }

    #[allow(dead_code)]
    pub fn get_index_or_insert(&mut self, vector: T) -> Result<u32, HallrError> {
        if !vector.is_finite() {
            return Err(HallrError::FloatNotFinite(format!(
                "The vector was not finite {vector:?}"
            )));
        }
        // try to get rid of the -0.0 value by adding 0.0
        let x: T::Scalar = vector.x() + T::Scalar::ZERO;
        let y: T::Scalar = vector.y() + T::Scalar::ZERO;
        let index = self
            .set
            .entry((x.to_bits(), y.to_bits()))
            .or_insert_with(|| {
                let new_index = self.vertices.len();
                self.vertices.push(vector);
                new_index as u32
            });
        Ok(*index)
    }
}

// TODO replace with dedup crate
#[allow(clippy::type_complexity)]
pub(crate) struct VertexDeduplicator3D<T: GenericVector3> {
    set: FxHashMap<
        (
            <T::Scalar as GenericScalar>::BitsType,
            <T::Scalar as GenericScalar>::BitsType,
            <T::Scalar as GenericScalar>::BitsType,
        ),
        u32,
    >,
    pub vertices: Vec<T>,
}

impl<T: GenericVector3> VertexDeduplicator3D<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            set: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            vertices: Vec::with_capacity(capacity),
        }
    }

    /// get a previously defined index, or insert the vertex and return the new index
    pub fn get_index_or_insert(&mut self, vector: T) -> Result<u32, HallrError> {
        if !vector.is_finite() {
            return Err(HallrError::FloatNotFinite(format!(
                "The vector was not finite ({vector:?})"
            )));
        }
        // try to get rid of the -0.0 value by adding 0.0
        let x: T::Scalar = vector.x() + T::Scalar::ZERO;
        let y: T::Scalar = vector.y() + T::Scalar::ZERO;
        let z: T::Scalar = vector.z() + T::Scalar::ZERO;

        let index = self
            .set
            .entry((x.to_bits(), y.to_bits(), z.to_bits()))
            .or_insert_with(|| {
                let new_index = self.vertices.len();
                self.vertices.push(vector);
                new_index as u32
            });
        Ok(*index)
    }

    /// inserts a vertex without de-dup checking
    pub fn insert_and_get_index(&mut self, vector: T) -> u32 {
        let index = self.vertices.len() as u32;
        self.vertices.push(vector);
        index
    }

    /// clear the hashset, effectively creating a new set of unique points
    pub fn clear_dedup_cache(&mut self) {
        self.set.clear()
    }
}

pub(crate) struct IndexDeduplicator<T: HasXYZ> {
    set: FxHashMap<u32, u32>,
    pub vertices: Vec<T>,
}

#[allow(dead_code)]
impl<T: HasXYZ> IndexDeduplicator<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            set: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            vertices: Vec::with_capacity(capacity),
        }
    }

    /// get a previously defined index, or insert the vertex and return the new index
    pub fn get_index_or_insert<F>(&mut self, old_index: u32, vertex: F) -> Result<u32, HallrError>
    where
        F: Fn() -> T,
    {
        let index = self.set.entry(old_index).or_insert_with(|| {
            let new_index = self.vertices.len();
            self.vertices.push(vertex());
            new_index as u32
        });
        Ok(*index)
    }

    /// inserts a vertex without de-dup checking
    pub fn get_index_and_insert(&mut self, vector: T) -> u32 {
        let index = self.vertices.len() as u32;
        self.vertices.push(vector);
        index
    }

    /// clear the hashset, effectively creating a new set of unique points
    pub fn clear_dedup_cache(&mut self) {
        self.set.clear()
    }
}

/// constructs the adjacency map for unordered edges.
#[allow(dead_code)]
#[allow(clippy::type_complexity)]
pub fn adjacency_map_from_unordered_edges(
    edges: &[usize],
) -> Result<(usize, FxHashMap<usize, SmallVec<[usize; 2]>>), HallrError> {
    let mut lowest_index = MaximumTracker::<Reverse<usize>>::default();

    if edges.len() < 2 {
        return Err(HallrError::InvalidParameter(
            "The line segment should have at least 2 vertices.".to_string(),
        ));
    }

    let mut adjacency: FxHashMap<usize, SmallVec<[usize; 2]>> =
        FxHashMap::with_capacity_and_hasher(edges.len(), Default::default());
    for chunk in edges.chunks(2) {
        let a = chunk[0];
        let b = chunk[1];
        lowest_index.insert(Reverse(a));
        lowest_index.insert(Reverse(b));

        adjacency.entry(a).or_default().push(b);
        adjacency.entry(b).or_default().push(a);

        // Check for more than two neighbors and handle error
        if adjacency.get(&a).unwrap().len() > 2 || adjacency.get(&b).unwrap().len() > 2 {
            return Err(HallrError::InvalidParameter(
                "More than two neighbors for a vertex in a loop.".to_string(),
            ));
        }
    }
    Ok((lowest_index.get_max().unwrap().0, adjacency))
}

/// Constructs a continuous loop of vertex indices from an unordered list of edges.
///
/// This function takes as input a slice of `usize` that represents edges by pairing
/// consecutive values. For example, a slice `[a, b, c, d]` represents two edges: `a-b` and `c-d`.
///
/// # Arguments
///
/// * `edges` - A slice of vertex indices, where each consecutive pair represents an edge.
///   The slice's length should be even.
///
/// # Returns
///
/// * If successful, a vector of vertex indices that forms a continuous loop.
/// * If unsuccessful, a `CollisionError` indicating the nature of the error.
///
/// # Example
///
/// ```rust,ignore
/// let edges = [1, 0, 2, 1, 3, 2, 0, 3];
/// let loop_indices = continuous_loop_from_unordered_edges(&edges)?;
/// assert_eq!(loop_indices, vec![1, 0, 3, 2, 1]);
/// ```
///
/// # Errors
///
/// This function may return an error in the following scenarios:
///
/// * The input edge list is malformed or does not form a valid loop.
/// * There are missing vertices in the adjacency map.
///
/// # Note
///
/// The function assumes that the input edge list is valid, i.e., forms a closed loop
/// without isolated vertices or unconnected components.
#[allow(dead_code)]
pub fn reconstruct_from_unordered_edges(edges: &[usize]) -> Result<Vec<usize>, HallrError> {
    if edges.len() < 2 {
        return Err(HallrError::InvalidParameter(
            "The line segment should have at least 2 vertices.".to_string(),
        ));
    }

    let (lowest_index, adjacency) = adjacency_map_from_unordered_edges(edges)?;

    // Detect endpoints (vertices with only one neighbor)
    let endpoints: Vec<_> = adjacency
        .iter()
        .filter(|(_, neighbors)| neighbors.len() == 1)
        .map(|(&vertex, _)| vertex)
        .collect();

    let is_loop = endpoints.is_empty();

    let mut current = if is_loop {
        // Start at lowest index for a loop
        lowest_index
    } else {
        // Start at one of the endpoints for a line
        endpoints[0].min(endpoints[1])
    };
    let starting_point = current;

    let mut visited = FxHashSet::default();
    let _ = visited.insert(current);
    let mut reconstructed = vec![current];

    let next_neighbors = &adjacency[&current];
    if (is_loop && next_neighbors.len() != 2) || (!is_loop && next_neighbors.len() > 1) {
        return Err(HallrError::InvalidParameter(
            "The provided line segment has more than two adjacent vertices.".to_string(),
        ));
    }

    if is_loop {
        current = next_neighbors[0].min(next_neighbors[1]);
    } else {
        current = next_neighbors[0]
    }
    reconstructed.push(current);
    let _ = visited.insert(current);
    loop {
        let next_neighbors: Vec<_> = adjacency[&current]
            .iter()
            .filter(|&n| !visited.contains(n))
            .collect();

        // Exit conditions
        if next_neighbors.is_empty() {
            break;
        }

        if next_neighbors.len() > 1 {
            return Err(HallrError::InvalidParameter(
                "The provided line segment have more than two adjacent vertices.".to_string(),
            ));
        }

        current = *next_neighbors[0];
        reconstructed.push(current);
        let _ = visited.insert(current);
    }
    // Add the starting point for a loop after the while loop.
    if is_loop {
        reconstructed.push(starting_point);
    }

    Ok(reconstructed)
}

#[allow(dead_code)]
pub(crate) trait UnsafeVob {
    /// unsafe (thorn) get()
    fn ᚦget(&self, index: u32) -> bool;
    /// unsafe (thorn) set()
    fn ᚦset(&mut self, bit: u32, flag: bool);
}

impl UnsafeVob for vob::Vob {
    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn ᚦget(&self, bit: u32) -> bool {
        unsafe { self.get_unchecked(bit as usize) }
    }

    #[cfg(debug_assertions)]
    #[inline(always)]
    fn ᚦget(&self, bit: u32) -> bool {
        self.get(bit as usize).unwrap()
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn ᚦset(&mut self, bit: u32, flag: bool) {
        unsafe {
            let _ = self.set_unchecked(bit as usize, flag);
        };
    }
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn ᚦset(&mut self, bit: u32, flag: bool) {
        unsafe {
            let _ = self.set_unchecked(bit as usize, flag);
        }
    }
}

pub(crate) trait UnsafeArray<T> {
    /// unsafe (thorn) get()
    fn ᚦget(&self, index: u32) -> &T;
    /// unsafe (thorn) get_mut()
    fn ᚦget_mut(&mut self, index: u32) -> &mut T;
}

impl<T> UnsafeArray<T> for [T] {
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn ᚦget(&self, index: u32) -> &T {
        self.get(index as usize).unwrap()
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn ᚦget(&self, index: u32) -> &T {
        unsafe { self.get_unchecked(index as usize) }
    }

    #[cfg(debug_assertions)]
    #[inline(always)]
    fn ᚦget_mut(&mut self, index: u32) -> &mut T {
        self.get_mut(index as usize).unwrap()
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn ᚦget_mut(&mut self, index: u32) -> &mut T {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl<T> UnsafeArray<T> for Vec<T> {
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn ᚦget(&self, index: u32) -> &T {
        self.get(index as usize).unwrap()
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn ᚦget(&self, index: u32) -> &T {
        unsafe { self.get_unchecked(index as usize) }
    }

    #[cfg(debug_assertions)]
    #[inline(always)]
    fn ᚦget_mut(&mut self, index: u32) -> &mut T {
        self.get_mut(index as usize).unwrap()
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn ᚦget_mut(&mut self, index: u32) -> &mut T {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

#[allow(dead_code)]
pub fn time_it<T>(label: impl Into<String>, work: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = work();
    let elapsed = start.elapsed();
    println!("{}: {:?}", label.into(), elapsed);
    result
}

#[allow(dead_code)]
pub fn time_it_r<T>(
    label: impl Into<String>,
    work: impl FnOnce() -> Result<T, HallrError>,
) -> Result<T, HallrError> {
    time_it(label, || -> Result<T, HallrError> { work() })
}
