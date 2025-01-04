use bevy::prelude::Event;
use bevy::prelude::*;

use crate::status_effects::StatusEffectType;

#[derive(Event)]
pub struct StatusEffectAppliedEvent {
    pub entity: Entity,
    pub effect: StatusEffectType,
}
