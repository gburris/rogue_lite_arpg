use bevy::prelude::*;

use super::components::StatusType;

#[derive(Event)]
pub struct ApplyEffects {
    pub effect_source: Entity,
}

impl ApplyEffects {}

#[derive(Event, Clone)]
pub struct ApplyStatus {
    pub status: StatusType,
    pub duration: f32,
}
