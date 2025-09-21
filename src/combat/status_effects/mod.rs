mod burn;
mod freeze;
mod slow;

pub use burn::Burning;
pub use freeze::Frozen;
pub use slow::Slowed;

use bevy::{ecs::entity_disabling::Disabled, prelude::*};

use crate::{labels::sets::InGameSet, utility::Lifespan};

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                burn::apply_burning,
                burn::tick_burn,
                burn::while_burning,
                freeze::apply_frozen,
                slow::apply_slowed,
            )
                .chain()
                .in_set(InGameSet::Simulation),
        )
        .add_observer(burn::on_burn_removed)
        .add_observer(freeze::on_frozen_removed)
        .add_observer(slow::on_slow_removed);
    }
}

#[derive(Component, Default, Clone)]
#[require(Lifespan)]
pub struct Status;

#[derive(Component, Clone)]
#[require(Disabled)]
#[relationship(relationship_target = Effects)]
pub struct EffectOf(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = EffectOf, linked_spawn)]
pub struct Effects(Vec<Entity>);

#[derive(Component, Clone)]
#[relationship(relationship_target = Statuses)]
pub struct StatusOf(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = StatusOf, linked_spawn)]
pub struct Statuses(Vec<Entity>);

#[derive(Component, Clone)]
pub struct StatusApplied;
