#[cfg(test)]
mod test;

use crate::{
    HallrError,
    command::{ConfigType, Model, Options},
    ffi,
    ffi::FFIVector3,
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;
use smallvec::SmallVec;
use vector_traits::glam::Vec3;
use crate::utils::UnsafeArray;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Edge {
    pub v0: usize,
    pub v1: usize,
}

impl Edge {
    pub fn new(v0: usize, v1: usize) -> Self {
        // Ensure consistent ordering
        if v0 < v1 {
            Self { v0, v1 }
        } else {
            Self { v0: v1, v1: v0 }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub v0: usize,
    pub v1: usize,
    pub v2: usize,
}

impl Face {
    pub fn new(v0: usize, v1: usize, v2: usize) -> Self {
        Self { v0, v1, v2 }
    }

    pub fn edges(&self) -> [Edge; 3] {
        [
            Edge::new(self.v0, self.v1),
            Edge::new(self.v1, self.v2),
            Edge::new(self.v2, self.v0),
        ]
    }

    pub fn normal(&self, vertices: &[Vec3]) -> Vec3 {
        let v0 = vertices[self.v0];
        let v1 = vertices[self.v1];
        let v2 = vertices[self.v2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        edge1.cross(edge2).normalize()
    }

    pub fn contains_vertex(&self, vertex_idx: usize) -> bool {
        self.v0 == vertex_idx || self.v1 == vertex_idx || self.v2 == vertex_idx
    }
}

#[derive(Debug, Clone)]
struct MeshAnalysis {
    non_manifold_edges: Vec<Edge>,
    non_manifold_vertices: Vec<usize>,
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<Face>,
    // Cached analysis - invalidated when mesh is modified
    cached_analysis: Option<MeshAnalysis>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vec3>, faces: Vec<Face>) -> Self {
        Self {
            vertices,
            faces,
            cached_analysis: None,
        }
    }

    /// Invalidate cached analysis when mesh is modified
    fn invalidate_cache(&mut self) {
        self.cached_analysis = None;
    }

    /// Get or compute the mesh analysis
    fn get_analysis(&mut self) -> &MeshAnalysis {
        if self.cached_analysis.is_none() {
            self.cached_analysis = Some(MeshAnalysis {
                non_manifold_edges: self.compute_non_manifold_edges(),
                non_manifold_vertices: self.compute_non_manifold_vertices(),
            });
        }
        self.cached_analysis.as_ref().unwrap()
    }

    /// Detect non-manifold edges (edges shared by faces with opposite normals)
    pub fn detect_non_manifold_edges(&mut self) -> &[Edge] {
        &self.get_analysis().non_manifold_edges
    }

    /// Detect non-manifold vertices (vertices that connect disconnected surface components)  
    pub fn detect_non_manifold_vertices(&mut self) -> &[usize] {
        &self.get_analysis().non_manifold_vertices
    }

    /// Internal computation method - separated from public API
    fn compute_non_manifold_edges(&self) -> Vec<Edge> {
        let mut edge_to_faces: FxHashMap<Edge, SmallVec<[usize;3]>> =
            FxHashMap::with_capacity_and_hasher(self.vertices.len(), Default::default());

        // Build edge-to-face mapping
        for (face_idx, face) in self.faces.iter().enumerate() {
            for edge in face.edges() {
                edge_to_faces
                    .entry(edge)
                    .or_default()
                    .push(face_idx);
            }
        }

        let mut non_manifold_edges = Vec::new();

        for (edge, face_indices) in edge_to_faces {
            if face_indices.len() >= 2 {
                // Check if any pair of faces has opposite normals
                for i in 0..face_indices.len() {
                    for j in i + 1..face_indices.len() {
                        let face1 = &self.faces[face_indices[i]];
                        let face2 = &self.faces[face_indices[j]];

                        let normal1 = face1.normal(&self.vertices);
                        let normal2 = face2.normal(&self.vertices);

                        // Check if normals are roughly opposite (dot product < -0.5)
                        if normal1.dot(normal2) < -0.5 {
                            non_manifold_edges.push(edge);
                            break;
                        }
                    }
                }
            }
        }

        non_manifold_edges
    }

    /// Internal computation method - separated from public API
    fn compute_non_manifold_vertices(&self) -> Vec<usize> {
        let mut non_manifold_vertices = Vec::new();

        // Build vertex-to-faces mapping
        let mut vertex_to_faces: FxHashMap<usize, Vec<usize>> =
            FxHashMap::with_capacity_and_hasher(self.vertices.len(), Default::default());

        for (face_idx, face) in self.faces.iter().enumerate() {
            vertex_to_faces
                .entry(face.v0)
                .or_default()
                .push(face_idx);
            vertex_to_faces
                .entry(face.v1)
                .or_default()
                .push(face_idx);
            vertex_to_faces
                .entry(face.v2)
                .or_default()
                .push(face_idx);
        }

        for (vertex_idx, face_indices) in vertex_to_faces {
            if face_indices.len() < 3 {
                continue; // Skip vertices with too few faces
            }

            // Check if the faces around this vertex form connected components
            let connected_components =
                self.get_face_components_around_vertex(vertex_idx, &face_indices);

            // If there are multiple disconnected components, this is a non-manifold vertex
            if connected_components.len() > 1 {
                // Additional check: ensure the components are actually spatially separated
                if self.are_components_spatially_separated(&connected_components) {
                    non_manifold_vertices.push(vertex_idx);
                }
            }
        }

        non_manifold_vertices
    }

    /// Get connected components of faces around a vertex
    fn get_face_components_around_vertex(
        &self,
        vertex_idx: usize,
        face_indices: &[usize],
    ) -> Vec<Vec<usize>> {
        let mut visited = FxHashSet::with_capacity_and_hasher(face_indices.len(), Default::default());
        let mut components = Vec::new();

        for &face_idx in face_indices {
            if visited.contains(&face_idx) {
                continue;
            }

            let mut component = Vec::new();
            let mut stack = vec![face_idx];

            while let Some(current_face) = stack.pop() {
                if visited.contains(&current_face) {
                    continue;
                }

                let _ = visited.insert(current_face);
                component.push(current_face);

                // Find adjacent faces that share an edge (not just the vertex)
                for &other_face_idx in face_indices {
                    if visited.contains(&other_face_idx) {
                        continue;
                    }

                    if self.faces_share_edge_through_vertex(
                        current_face,
                        other_face_idx,
                        vertex_idx,
                    ) {
                        stack.push(other_face_idx);
                    }
                }
            }

            if !component.is_empty() {
                components.push(component);
            }
        }

        components
    }

    /// Check if two faces share an edge that includes the given vertex
    fn faces_share_edge_through_vertex(
        &self,
        face1_idx: usize,
        face2_idx: usize,
        vertex_idx: usize,
    ) -> bool {
        let face1 = &self.faces[face1_idx];
        let face2 = &self.faces[face2_idx];

        let face1_edges = face1.edges();
        let face2_edges = face2.edges();

        for edge1 in &face1_edges {
            if edge1.v0 != vertex_idx && edge1.v1 != vertex_idx {
                continue; // This edge doesn't involve our vertex
            }

            for edge2 in &face2_edges {
                if edge1 == edge2 {
                    return true; // Shared edge found
                }
            }
        }

        false
    }

    /// Check if face components around a vertex are spatially separated
    fn are_components_spatially_separated(
        &self,
        components: &[Vec<usize>],
    ) -> bool {
        if components.len() < 2 {
            return false;
        }

        // Calculate average normals for each component
        let mut component_normals = Vec::new();

        for component in components {
            let mut avg_normal = Vec3::ZERO;
            let mut count = 0;

            for &face_idx in component {
                let face = &self.faces[face_idx];
                let normal = face.normal(&self.vertices);
                if normal.length() > 0.0 {
                    avg_normal += normal;
                    count += 1;
                }
            }

            if count > 0 {
                avg_normal /= count as f32;
                avg_normal = avg_normal.normalize();
                component_normals.push(avg_normal);
            }
        }

        // Check if component normals are significantly different
        // (indicating different surface orientations)
        for i in 0..component_normals.len() {
            for j in i + 1..component_normals.len() {
                let dot_product = component_normals.u_get(i).dot(*component_normals.u_get(j));
                if dot_product < 0.5 {
                    // Normals differ by more than 60 degrees
                    return true;
                }
            }
        }

        false
    }

    /// Fix non-manifold vertices by duplicating them for each connected component
    pub fn fix_non_manifold_vertices(&mut self) -> usize {
        // Get the current non-manifold vertices (this will cache the analysis)
        let non_manifold_vertices: Vec<usize> = self.detect_non_manifold_vertices().to_vec();
        let mut fixes_applied = 0;

        for vertex_idx in non_manifold_vertices {
            if self.split_non_manifold_vertex(vertex_idx) {
                fixes_applied += 1;
            }
        }

        // Invalidate cache since we modified the mesh
        if fixes_applied > 0 {
            self.invalidate_cache();
        }

        fixes_applied
    }

    /// Split a non-manifold vertex into multiple vertices
    fn split_non_manifold_vertex(&mut self, vertex_idx: usize) -> bool {
        // Get faces that use this vertex
        let mut vertex_faces = Vec::new();
        for (face_idx, face) in self.faces.iter().enumerate() {
            if face.contains_vertex(vertex_idx) {
                vertex_faces.push(face_idx);
            }
        }

        if vertex_faces.len() < 3 {
            return false; // Not enough faces to be problematic
        }

        // Get connected components
        let components = self.get_face_components_around_vertex(vertex_idx, &vertex_faces);

        if components.len() <= 1 {
            return false; // No splitting needed
        }

        let original_vertex_pos = self.vertices[vertex_idx];

        // Keep the first component using the original vertex
        // Create new vertices for other components
        for (comp_idx, component) in components.iter().enumerate().skip(1) {
            let new_vertex_idx = self.vertices.len();

            // Add slightly offset vertex to avoid exact duplicates
            let offset = Vec3::new(
                (comp_idx as f32) * 1e-6,
                (comp_idx as f32) * 1e-6,
                (comp_idx as f32) * 1e-6,
            );
            self.vertices.push(original_vertex_pos + offset);

            // Update faces in this component to use the new vertex
            for &face_idx in component {
                let face = self.faces.u_get_mut(face_idx);
                if face.v0 == vertex_idx {
                    face.v0 = new_vertex_idx;
                }
                if face.v1 == vertex_idx {
                    face.v1 = new_vertex_idx;
                }
                if face.v2 == vertex_idx {
                    face.v2 = new_vertex_idx;
                }
            }
        }

        true
    }

    /// Fix non-manifold edges by collapsing them to a single point
    pub fn fix_non_manifold_edges(&mut self) -> usize {
        // Get the current non-manifold edges (this will cache the analysis)
        let non_manifold_edges: Vec<Edge> = self.detect_non_manifold_edges().to_vec();
        let mut fixes_applied = 0;

        for edge in non_manifold_edges {
            if self.collapse_edge(edge) {
                fixes_applied += 1;
            }
        }

        // Remove degenerate faces and unused vertices
        self.cleanup();

        // Invalidate cache since we modified the mesh
        if fixes_applied > 0 {
            self.invalidate_cache();
        }

        fixes_applied
    }

    /// Collapse an edge by merging its two vertices
    fn collapse_edge(&mut self, edge: Edge) -> bool {
        let v0_idx = edge.v0;
        let v1_idx = edge.v1;

        if v0_idx >= self.vertices.len() || v1_idx >= self.vertices.len() {
            return false;
        }

        // Calculate midpoint
        let v0 = self.vertices.u_get(v0_idx);
        let v1 = self.vertices.u_get(v1_idx);
        let midpoint = Vec3 {
            x: (v0.x + v1.x) * 0.5,
            y: (v0.y + v1.y) * 0.5,
            z: (v0.z + v1.z) * 0.5,
        };

        // Update vertex position
        *self.vertices.u_get_mut(v0_idx) = midpoint;

        // Replace all references to v1_idx with v0_idx in faces
        for face in &mut self.faces {
            if face.v0 == v1_idx {
                face.v0 = v0_idx;
            }
            if face.v1 == v1_idx {
                face.v1 = v0_idx;
            }
            if face.v2 == v1_idx {
                face.v2 = v0_idx;
            }
        }

        true
    }

    /// Remove degenerate faces and compact vertex array
    fn cleanup(&mut self) {
        // Remove degenerate faces (faces with duplicate vertices)
        self.faces
            .retain(|face| face.v0 != face.v1 && face.v1 != face.v2 && face.v2 != face.v0);

        // Find used vertices
        let mut used_vertices: FxHashSet<usize> =
            FxHashSet::with_capacity_and_hasher(self.vertices.len(), Default::default());
        for face in &self.faces {
            let _ = used_vertices.insert(face.v0);
            let _ = used_vertices.insert(face.v1);
            let _ = used_vertices.insert(face.v2);
        }

        // Create vertex remapping
        let mut old_to_new: FxHashMap<usize, usize> =
            FxHashMap::with_capacity_and_hasher(self.vertices.len(), Default::default());
        let mut new_vertices = Vec::new();

        for (new_idx, &old_idx) in used_vertices.iter().enumerate() {
            let _ = old_to_new.insert(old_idx, new_idx);
            new_vertices.push(self.vertices[old_idx]);
        }

        // Update face indices
        for face in &mut self.faces {
            face.v0 = old_to_new[&face.v0];
            face.v1 = old_to_new[&face.v1];
            face.v2 = old_to_new[&face.v2];
        }

        self.vertices = new_vertices;
    }

    /// Fix all non-manifold issues iteratively until convergence
    pub fn fix_non_manifold_iterative(&mut self, max_iterations: usize) -> (usize, usize) {
        let mut total_vertex_fixes = 0;
        let mut total_edge_fixes = 0;

        for iteration in 0..max_iterations {
            let vertex_fixes = self.fix_non_manifold_vertices();
            let edge_fixes = self.fix_non_manifold_edges();

            total_vertex_fixes += vertex_fixes;
            total_edge_fixes += edge_fixes;

            println!(
                "Rust: Iteration {}: {} vertex fixes, {} edge fixes",
                iteration + 1,
                vertex_fixes,
                edge_fixes
            );

            // If no fixes were applied in this iteration, we're done
            if vertex_fixes == 0 && edge_fixes == 0 {
                println!("Rust: Converged after {} iterations", iteration + 1);
                break;
            }
        }

        (total_vertex_fixes, total_edge_fixes)
    }

    /// Get mesh statistics - now much more efficient with caching
    pub fn stats(&mut self) -> (usize, usize, usize, usize) {
        let vertices_len = self.vertices.len();
        let faces_len = self.faces.len();
        
        let analysis = self.get_analysis();
        (
            vertices_len,
            faces_len,
            analysis.non_manifold_edges.len(),
            analysis.non_manifold_vertices.len(),
        )
    }
}

pub(crate) fn process_command(
    input_config: ConfigType,
    models: Vec<Model<'_>>,
) -> Result<super::CommandResult, HallrError> {
    if models.len() != 1 {
        Err(HallrError::InvalidInputData(
            "Rust: Incorrect number of models selected".to_string(),
        ))?
    }
    input_config.confirm_mesh_packaging(0, ffi::MeshFormat::Triangulated)?;
    let model = &models[0];
    let world_matrix = model.world_orientation.to_vec();
    let max_iterations = input_config
        .get_parsed_option::<usize>("max_iterations")?
        .unwrap_or(5);

    let vertices: Vec<Vec3> = model.vertices.iter().map(|v| v.into()).collect::<Vec<_>>();
    let indices = model
        .indices
        .chunks_exact(3)
        .map(|i| Face::new(i[0], i[1], i[2]))
        .collect();

    println!("Rust: mesh cleanup starting");
    let start = Instant::now();
    let mut mesh = Mesh::new(vertices, indices);

    // Detect and report initial issues
    let (initial_vertices, initial_faces, initial_bad_edges, initial_bad_vertices) = mesh.stats();
    println!(
        "Rust: Initial mesh stats: {initial_vertices} vertices, {initial_faces} faces, {initial_bad_edges} non-manifold edges, {initial_bad_vertices} non-manifold vertices",
    );

    // Fix non-manifold vertices first (your SDF artifact issue)
    let (vertex_fixes, edge_fixes) = mesh.fix_non_manifold_iterative(max_iterations);
    println!("Rust: Applied {vertex_fixes} vertex fixes");

    println!("Rust: Applied {edge_fixes} edge fixes");

    // Report final stats
    let (final_vertices, final_faces, final_bad_edges, final_bad_vertices) = mesh.stats();
    println!(
        "Rust: Final mesh stats: {final_vertices} vertices, {final_faces} faces, {final_bad_edges} non-manifold edges, {final_bad_vertices} non-manifold vertices"
    );

    println!("Rust: mesh::fix() execution time {:?}", start.elapsed());

    // Get the final vertex array
    let mut ffi_vertices: Vec<FFIVector3> = mesh.vertices.iter().map(|v| (*v).into()).collect();
    let indices: Vec<usize> = mesh
        .faces
        .iter()
        .flat_map(|f| [f.v0, f.v1, f.v2])
        .collect();

    if let Some(world_to_local) = model.get_world_to_local_transform()? {
        // Transform to local
        println!(
            "Rust: applying world-local transformation 1/{:?}",
            model.world_orientation
        );
        ffi_vertices
            .iter_mut()
            .for_each(|v| *v = world_to_local(*v));
    } else {
        println!("Rust: *not* applying world-local transformation");
    }

    let mut return_config = ConfigType::new();
    let _ = return_config.insert(
        ffi::MeshFormat::MESH_FORMAT_TAG.to_string(),
        ffi::MeshFormat::Triangulated.to_string(),
    );

    Ok((ffi_vertices, indices, world_matrix, return_config))
}
