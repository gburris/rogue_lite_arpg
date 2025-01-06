use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;

#[derive(Event)]
pub struct AttemptDialogueInput;

#[derive(Event)]
pub struct DialogueBegin {
    pub entity: Entity,
    pub colliding_entities: CollidingEntities,
}
