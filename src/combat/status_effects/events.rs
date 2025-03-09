use bevy::prelude::*;

use super::components::StatusType;

#[derive(Event)]
pub struct ApplyEffect {
    pub effect: Vec<ApplyStatus>,
}

#[derive(Event, Debug, Clone)]
pub struct ApplyStatus {
    pub status: StatusType,
    pub duration: f32,
}
