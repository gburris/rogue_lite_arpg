use bevy::prelude::*;

#[derive(Event)]
pub struct DialogueBegin {
    pub entity: Entity,
}
