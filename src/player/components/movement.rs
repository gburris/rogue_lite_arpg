use bevy::prelude::*;

#[derive(Component, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum MovementDirection {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl MovementDirection {
    pub fn from_vec2(vec: Vec2) -> Self {
        match vec.normalize() {
            v if v.y > 0.5 => Self::Up,
            v if v.y < -0.5 => Self::Down,
            v if v.x > 0.5 => Self::Right,
            v if v.x < -0.5 => Self::Left,
            _ => Self::None,
        }
    }
}
