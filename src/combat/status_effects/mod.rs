mod burn;
mod freeze;
mod slow;

pub mod prelude {
    pub use super::Effects;
    pub use super::burn::Burning;
    pub use super::freeze::Frozen;
    pub(crate) use super::slow::sync_slow_from_statuses;
}

use bevy::ecs::entity_disabling::Disabled;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::prelude::{InGameSystems, Lifespan};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(apply_effects);

    app.add_systems(
        Update,
        (
            burn::apply_burning,
            (burn::tick_burn, burn::while_burning).chain(),
            freeze::apply_frozen,
            slow::sync_slow_from_statuses,
        )
            .in_set(InGameSystems::Simulation),
    );
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
pub(crate) struct StatusApplied;

#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, Debug)]
enum StatusType {
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

#[derive(Event)]
pub struct ApplyEffects {
    effects: Vec<Entity>,
    target: Entity,
}

impl ApplyEffects {
    pub fn new(effects: &Effects, target: Entity) -> Self {
        Self {
            effects: effects.0.clone(),
            target,
        }
    }
}

fn apply_effects(
    apply_effects: On<ApplyEffects>,
    mut commands: Commands,
    effect_query: Query<&StatusType, (Without<StatusOf>, Allow<Disabled>)>,
    affected_query: Query<&Statuses>,
    status_query: Query<&StatusType, With<StatusOf>>,
) {
    let mut new_status_types = HashSet::new();
    let mut deduped_effects = Vec::new();
    for &entity in &apply_effects.effects {
        let Ok(status_type) = effect_query.get(entity) else {
            warn!("Entity {entity:?} missing StatusType component, skipping it");
            continue;
        };
        if new_status_types.insert(status_type) {
            deduped_effects.push(entity);
        }
    }

    // Remove any existing statuses on the target that conflict with new effects
    if let Ok(statuses) = affected_query.get(apply_effects.target) {
        for existing_entity in statuses.iter() {
            if let Ok(existing_type) = status_query.get(existing_entity)
                && new_status_types.contains(existing_type)
            {
                debug!("Removing existing status");
                commands.entity(existing_entity).despawn();
            }
        }
    }

    debug!("Applying effects: {:?}", deduped_effects);
    for effect in deduped_effects {
        commands
            .entity(effect)
            .clone_and_spawn()
            .remove::<(Disabled, EffectOf)>()
            .insert(StatusOf(apply_effects.target));
    }
}
