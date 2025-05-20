use crate::command::cmd_lsystems::lsystems::Turtle;
use vector_traits::glam::{DQuat, DVec4};

impl Default for Turtle {
    fn default() -> Self {
        Self {
            orientation: DQuat::IDENTITY,
            position: DVec4::ZERO,
            result: Vec::default(),
            stack: Vec::default(),
            pen_up: false,
            round: false,
            sphere_radius: 1.0,
        }
    }
}
