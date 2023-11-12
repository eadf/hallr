// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::{command::cmd_voronoi_mesh::DiagramHelperRw, utils::VertexDeduplicator3D};
use vector_traits::GenericVector3;

impl<T: GenericVector3> Default for DiagramHelperRw<T> {
    fn default() -> Self {
        Self {
            vertex_map: VertexDeduplicator3D::<T>::default(),
        }
    }
}
