mod burn;
mod freeze;
mod slow;

pub mod prelude {
    pub use super::burn::*;
    pub use super::freeze::*;
    pub use super::{EffectOf, Effects, StatusOf};
}

use bevy::{ecs::entity_disabling::Disabled, prelude::*};

use crate::prelude::{InGameSystems, Lifespan};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            burn::apply_burning,
            (burn::tick_burn, burn::while_burning).chain(),
            freeze::apply_frozen,
            slow::apply_slowed,
        )
            .in_set(InGameSystems::Simulation),
    )
    .add_observer(slow::on_slow_removed);
}

#[derive(Component, Clone)]
#[require(Disabled)]
#[relationship(relationship_target = Effects)]
pub struct EffectOf(Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = EffectOf, linked_spawn)]
pub struct Effects(Vec<Entity>);

#[derive(Component, Clone)]
#[relationship(relationship_target = Statuses)]
#[require(Lifespan)]
pub struct StatusOf(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = StatusOf, linked_spawn)]
pub struct Statuses(Vec<Entity>);

#[derive(Component, Clone)]
struct StatusApplied;
