use crate::components::position::Position;
use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub speed: f32,
    pub position: Position,
}
