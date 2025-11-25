mod burn;
mod freeze;
mod slow;

pub mod prelude {
    pub use super::burn::*;
    pub use super::freeze::*;
    pub use super::{EffectOf, Effects, StatusOf};
}

use bevy::{ecs::entity_disabling::Disabled, platform::collections::HashSet, prelude::*};

use crate::prelude::{InGameSystems, Lifespan};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(apply_effects);

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

#[derive(Component, Clone, Hash, Eq, PartialEq, Debug)]
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
    let deduped_effects: Vec<Entity> = apply_effects
        .effects
        .iter()
        .filter_map(|&entity| {
            if let Ok(status_type) = effect_query.get(entity) {
                if new_status_types.insert(status_type) {
                    Some(entity)
                } else {
                    None // Duplicate status type, filter it out
                }
            } else {
                // If we can't get the StatusType, keep the entity but warn
                warn!("Entity {:?} missing StatusType component", entity);
                None
            }
        })
        .collect();

    // Remove any existing statuses on the target that conflict with new effects
    if let Ok(statuses) = affected_query.get(apply_effects.target) {
        for existing_entity in statuses.iter() {
            if let Ok(existing_type) = status_query.get(existing_entity) {
                if new_status_types.contains(existing_type) {
                    debug!("Removing existing status");
                    commands.entity(existing_entity).despawn();
                }
            }
        }
    }

    debug!("Applying effects: {:?}", deduped_effects);
    deduped_effects.iter().for_each(|&effect| {
        commands
            .entity(effect)
            .clone_and_spawn()
            .remove::<(Disabled, EffectOf)>()
            .insert(StatusOf(apply_effects.target));
    });
}
