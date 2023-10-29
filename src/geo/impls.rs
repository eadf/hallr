use super::*;
use std::hash::{Hash, Hasher};
use vector_traits::glam::{DVec2, Vec2};

impl PartialEq for HashableVector2 {
    fn eq(&self, other: &Self) -> bool {
        ulps_eq!(self, other)
    }
}

impl Eq for HashableVector2 {}

impl Hash for HashableVector2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl From<HashableVector2> for Vec2 {
    fn from(hashable: HashableVector2) -> Self {
        Self {
            x: hashable.x,
            y: hashable.y,
        }
    }
}

impl From<Vec2> for HashableVector2 {
    fn from(vec: Vec2) -> Self {
        HashableVector2 { x: vec.x, y: vec.y }
    }
}

impl From<HashableVector2> for DVec2 {
    fn from(hashable: HashableVector2) -> Self {
        Self {
            x: hashable.x as f64,
            y: hashable.y as f64,
        }
    }
}

impl From<DVec2> for HashableVector2 {
    fn from(vec: DVec2) -> Self {
        HashableVector2 {
            x: vec.x as f32,
            y: vec.y as f32,
        }
    }
}

impl AbsDiffEq for HashableVector2 {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        f32::abs_diff_eq(&self.x, &other.x, epsilon) && f32::abs_diff_eq(&self.y, &other.y, epsilon)
    }
}

impl UlpsEq for HashableVector2 {
    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        f32::ulps_eq(&self.x, &other.x, epsilon, max_ulps)
            && f32::ulps_eq(&self.y, &other.y, epsilon, max_ulps)
    }
}
