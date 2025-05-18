// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use std::fmt::Debug;
use vector_traits::glam::{DVec2, DVec3};

#[allow(dead_code)]
const EPSILON: f64 = 1e-10; // or some small value that's appropriate for your use case
#[allow(dead_code)]
fn assert_approx_eq<T: SillyApproxEq + Debug>(v1: T, v2: T, epsilon: f64) {
    assert!(v1.silly_approx_eq(&v2, epsilon), "{v1:?} != {v2:?}");
}

trait SillyApproxEq {
    fn silly_approx_eq(&self, other: &Self, epsilon: f64) -> bool;
}

impl SillyApproxEq for f64 {
    fn silly_approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        (self - other).abs() <= epsilon
    }
}

impl SillyApproxEq for DVec2 {
    fn silly_approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() <= epsilon && (self.y - other.y).abs() <= epsilon
    }
}

impl SillyApproxEq for DVec3 {
    fn silly_approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() <= epsilon
            && (self.y - other.y).abs() <= epsilon
            && (self.z - other.z).abs() <= epsilon
    }
}
