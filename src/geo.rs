mod impls;

use std::fmt::Debug;
use vector_traits::approx::*;

#[derive(Clone, Copy, Debug)]
pub struct HashableVector2 {
    x: f32,
    y: f32,
}
impl HashableVector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
