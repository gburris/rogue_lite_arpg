use bevy::prelude::*;

use crate::prelude::Lifespan;

#[derive(Component, Clone)]
#[relationship(relationship_target = Statuses)]
#[require(Lifespan)]
pub struct StatusOf(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = StatusOf, linked_spawn)]
pub struct Statuses(Vec<Entity>);

#[derive(Component, Clone)]
pub(crate) struct StatusApplied;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum StatusType {
    Burn,
    Freeze,
    Slow,
}

#[derive(Component, Clone)]
#[relationship(relationship_target = StatusVisualEffects)]
pub(super) struct StatusVfxOf(pub(super) Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = StatusVfxOf, linked_spawn)]
pub(super) struct StatusVisualEffects(Vec<Entity>);
