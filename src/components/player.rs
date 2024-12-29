use bevy::prelude::*;
use crate::components::position::Position;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub position: Position,
}