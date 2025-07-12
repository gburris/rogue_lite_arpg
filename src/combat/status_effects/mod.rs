pub mod components;
mod status_systems;

use bevy::prelude::*;

use crate::{combat::status_effects::status_systems::*, labels::sets::InGameSet};

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (burning::tick_burn, burning::while_burning)
                .chain()
                .in_set(InGameSet::Simulation),
        )
        .add_observer(burning::on_burn_applied)
        .add_observer(burning::on_burn_removed)
        .add_observer(frozen::on_frozen_applied)
        .add_observer(frozen::on_frozen_removed)
        .add_observer(slowed::on_slow_applied)
        .add_observer(slowed::on_slow_removed);
    }
}

#[derive(Component)]
#[relationship(relationship_target = Effects)]
pub struct EffectOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = EffectOf)]
pub struct Effects(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Statuses)]
pub struct StatusOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = StatusOf)]
pub struct Statuses(Vec<Entity>);
