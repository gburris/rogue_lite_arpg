use bevy::prelude::*;

#[derive(Event)]
pub struct PlayerMovementEvent {
    pub direction: Vec2,
}
