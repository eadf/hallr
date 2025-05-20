// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

//! A module containing boilerplate implementations of standard traits such as Default, From etc etc

use super::VertexDeduplicator3D;
use ahash::AHashMap;
use vector_traits::prelude::GenericVector3;

// for some reason the derived Default impl requires T to be Default
impl<T: GenericVector3> Default for VertexDeduplicator3D<T> {
    fn default() -> Self {
        Self {
            set: AHashMap::default(),
            vertices: Vec::default(),
        }
    }
}
