// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2023, 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

//! A module containing boilerplate implementations of standard traits such as Default, From etc etc

use super::{FFIVector3, MeshFormat};
use baby_shark::exports::nalgebra;
use hronn::prelude::ConvertTo;
use std::fmt;

use crate::HallrError;
use vector_traits::{
    approx::{AbsDiffEq, UlpsEq},
    glam::{DVec3, Vec3, Vec3A, dvec3, vec3, vec3a},
    prelude::{Approx, GenericScalar, HasXY, HasXYZ},
};

impl fmt::Debug for FFIVector3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_float(value: f32) -> String {
            if value.fract() == 0.0 {
                format!("{value:.1}",)
            } else {
                format!("{value}",)
            }
        }

        write!(
            f,
            "({},{},{})",
            format_float(self.x),
            format_float(self.y),
            format_float(self.z)
        )
    }
}

impl fmt::Display for FFIVector3 {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl HasXY for FFIVector3 {
    type Scalar = f32;
    #[inline(always)]
    fn new_2d(x: Self::Scalar, y: Self::Scalar) -> Self {
        FFIVector3::new(x, y, Self::Scalar::ZERO)
    }
    #[inline(always)]
    fn x(self) -> Self::Scalar {
        self.x
    }
    #[inline(always)]
    fn x_mut(&mut self) -> &mut Self::Scalar {
        &mut self.x
    }
    #[inline(always)]
    fn set_x(&mut self, value: Self::Scalar) {
        self.x = value
    }
    #[inline(always)]
    fn y(self) -> Self::Scalar {
        self.y
    }
    #[inline(always)]
    fn y_mut(&mut self) -> &mut Self::Scalar {
        &mut self.y
    }
    #[inline(always)]
    fn set_y(&mut self, value: Self::Scalar) {
        self.y = value
    }
}

impl HasXYZ for FFIVector3 {
    #[inline(always)]
    fn new_3d(x: Self::Scalar, y: Self::Scalar, z: Self::Scalar) -> Self {
        Self { x, y, z }
    }
    #[inline(always)]
    fn z(self) -> Self::Scalar {
        self.z
    }
    #[inline(always)]
    fn z_mut(&mut self) -> &mut Self::Scalar {
        &mut self.z
    }
    #[inline(always)]
    fn set_z(&mut self, value: Self::Scalar) {
        self.z = value
    }
}

impl From<DVec3> for FFIVector3 {
    #[inline(always)]
    fn from(v: DVec3) -> Self {
        Self {
            x: v.x as f32,
            y: v.y as f32,
            z: v.z as f32,
        }
    }
}

impl From<Vec3> for FFIVector3 {
    #[inline(always)]
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<(f32, f32, f32)> for FFIVector3 {
    #[inline(always)]
    fn from(v: (f32, f32, f32)) -> Self {
        Self {
            x: v.0,
            y: v.1,
            z: v.2,
        }
    }
}

impl From<[f32;3]> for FFIVector3 {
    #[inline(always)]
    fn from(v: [f32;3]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
        }
    }
}

impl From<FFIVector3> for DVec3 {
    #[inline(always)]
    fn from(v: FFIVector3) -> Self {
        dvec3(v.x as f64, v.y as f64, v.z as f64)
    }
}

impl From<&FFIVector3> for DVec3 {
    #[inline(always)]
    fn from(v: &FFIVector3) -> Self {
        dvec3(v.x as f64, v.y as f64, v.z as f64)
    }
}

impl From<FFIVector3> for Vec3 {
    #[inline(always)]
    fn from(v: FFIVector3) -> Self {
        vec3(v.x, v.y, v.z)
    }
}

impl From<&FFIVector3> for Vec3 {
    #[inline(always)]
    fn from(v: &FFIVector3) -> Self {
        vec3(v.x, v.y, v.z)
    }
}

impl From<&FFIVector3> for Vec3A {
    #[inline(always)]
    fn from(v: &FFIVector3) -> Self {
        vec3a(v.x, v.y, v.z)
    }
}

impl From<FFIVector3> for Vec3A {
    #[inline(always)]
    fn from(v: FFIVector3) -> Self {
        vec3a(v.x, v.y, v.z)
    }
}

impl ConvertTo<DVec3> for FFIVector3 {
    #[inline(always)]
    fn to(self) -> DVec3 {
        dvec3(self.x as f64, self.y as f64, self.z as f64)
    }
}

impl ConvertTo<Vec3> for FFIVector3 {
    #[inline(always)]
    fn to(self) -> Vec3 {
        vec3(self.x, self.y, self.z)
    }
}

impl ConvertTo<Vec3A> for FFIVector3 {
    #[inline(always)]
    fn to(self) -> Vec3A {
        vec3a(self.x, self.y, self.z)
    }
}

impl ConvertTo<FFIVector3> for Vec3A {
    #[inline(always)]
    fn to(self) -> FFIVector3 {
        FFIVector3::new(self.x, self.y, self.z)
    }
}

impl std::ops::Add for FFIVector3 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<'b> std::ops::Add<&'b FFIVector3> for &FFIVector3 {
    type Output = FFIVector3;

    #[inline(always)]
    fn add(self, rhs: &'b FFIVector3) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Div<f32> for FFIVector3 {
    type Output = Self;

    #[inline(always)]
    fn div(self, scalar: f32) -> Self::Output {
        Self::Output {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl std::ops::Sub for FFIVector3 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl UlpsEq for FFIVector3 {
    #[inline(always)]
    fn default_max_ulps() -> u32 {
        // Delegates to f32's default max ulps
        f32::default_max_ulps()
    }

    #[inline(always)]
    fn ulps_eq(&self, other: &Self, epsilon: f32, max_ulps: u32) -> bool {
        // Delegates to f32's ulps_eq for each component
        self.x.ulps_eq(&other.x, epsilon, max_ulps)
            && self.y.ulps_eq(&other.y, epsilon, max_ulps)
            && self.z.ulps_eq(&other.z, epsilon, max_ulps)
    }
}
impl AbsDiffEq for FFIVector3 {
    type Epsilon = f32;

    #[inline(always)]
    fn default_epsilon() -> Self::Epsilon {
        // Delegates to f32's default epsilon
        f32::default_epsilon()
    }

    #[inline(always)]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        // Delegates to f32's abs_diff_eq for each component
        self.x.abs_diff_eq(&other.x, epsilon)
            && self.y.abs_diff_eq(&other.y, epsilon)
            && self.z.abs_diff_eq(&other.z, epsilon)
    }
}

impl Approx for FFIVector3 {
    #[inline(always)]
    fn is_ulps_eq(
        self,
        other: Self,
        epsilon: <Self::Scalar as AbsDiffEq>::Epsilon,
        max_ulps: u32,
    ) -> bool {
        self.x.ulps_eq(&other.x, epsilon, max_ulps) && self.y.ulps_eq(&other.y, epsilon, max_ulps)
    }
    #[inline(always)]
    fn is_abs_diff_eq(self, other: Self, epsilon: <Self::Scalar as AbsDiffEq>::Epsilon) -> bool {
        self.x.abs_diff_eq(&other.x, epsilon) && self.y.abs_diff_eq(&other.y, epsilon)
    }
}

impl ConvertTo<FFIVector3> for Vec3 {
    #[inline(always)]
    fn to(self) -> FFIVector3 {
        FFIVector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl ConvertTo<FFIVector3> for DVec3 {
    #[inline(always)]
    fn to(self) -> FFIVector3 {
        FFIVector3 {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
        }
    }
}

impl From<FFIVector3> for nalgebra::Vector3<f32> {
    #[inline(always)]
    fn from(v: FFIVector3) -> nalgebra::Vector3<f32> {
        nalgebra::Vector3::new(v.x, v.y, v.z)
    }
}

impl From<&FFIVector3> for nalgebra::Vector3<f32> {
    #[inline(always)]
    fn from(v: &FFIVector3) -> nalgebra::Vector3<f32> {
        nalgebra::Vector3::new(v.x, v.y, v.z)
    }
}

impl From<nalgebra::Vector3<f32>> for FFIVector3 {
    #[inline(always)]
    fn from(v: nalgebra::Vector3<f32>) -> Self {
        FFIVector3::new(v.x, v.y, v.z)
    }
}

impl From<&nalgebra::Vector3<f32>> for FFIVector3 {
    #[inline(always)]
    fn from(v: &nalgebra::Vector3<f32>) -> Self {
        FFIVector3::new(v.x, v.y, v.z)
    }
}

impl From<FFIVector3> for [f32; 3] {
    #[inline(always)]
    fn from(v: FFIVector3) -> Self {
        [v.x, v.y, v.z]
    }
}

impl fmt::Display for MeshFormat {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl TryFrom<&str> for MeshFormat {
    type Error = HallrError;

    fn try_from(s: &str) -> Result<Self, HallrError> {
        // Extract the first character if present
        s.chars()
            .next()
            .ok_or_else(|| {
                HallrError::InvalidInputData("Empty string for MeshFormat conversion".to_string())
            })
            .and_then(MeshFormat::from_char)
    }
}
