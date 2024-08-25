use bevy::prelude::*;

use crate::pixel::cell::{Cell, PhysicsType};

pub const PARTICLE_GRAVITY: f32 = 0.1;

#[derive(Component, Clone, Copy, Debug)]
pub struct Particle {
    pub color: [u8; 4],
    pub physics: PhysicsType,

    pub velocity: Vec2,
}

impl From<Cell> for Particle {
    fn from(value: Cell) -> Self {
        Self {
            color: value.color,
            physics: value.physics,
            velocity: Vec2::ZERO,
        }
    }
}

impl Particle {
    pub fn from_cell_with_velocity_position(cell: &Cell, velocity: Vec2) -> Self {
        Self {
            color: cell.color,
            physics: cell.physics,
            velocity,
        }
    }
}