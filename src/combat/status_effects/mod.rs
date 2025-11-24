mod burn;
mod freeze;
mod slow;

pub mod prelude {
    pub use super::burn::*;
    pub use super::freeze::*;
    pub use super::{EffectOf, Effects, StatusOf};
}

use bevy::{
    ecs::entity_disabling::Disabled, platform::collections::HashMap, prelude::*,
    render::render_resource::encase::private::Length,
};

use crate::prelude::{InGameSystems, Lifespan};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        dedupe_statuses.in_set(InGameSystems::DespawnEntities),
    );

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

fn dedupe_statuses(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusType, &Lifespan, &StatusOf), Without<StatusApplied>>,
    active_status_query: Query<&StatusType, With<StatusApplied>>,
    affected_query: Query<&Statuses>,
) {
    // Group new statuses by target entity and status type
    let new_statuses_by_target = status_query.iter().fold(
        HashMap::<Entity, HashMap<StatusType, Vec<(Entity, Lifespan)>>>::new(),
        |mut acc, (status_entity, status_type, lifespan, status_of)| {
            acc.entry(status_of.0)
                .or_default()
                .entry(status_type.clone())
                .or_default()
                .push((status_entity, lifespan.clone()));
            acc
        },
    );

    // For each target entity, process each status type
    for (target_entity, new_statuses_by_type) in new_statuses_by_target {
        let statuses = match affected_query.get(target_entity) {
            Ok(statuses) => statuses,
            Err(_) => continue,
        };

        let existing_status_by_type: HashMap<StatusType, Entity> = statuses
            .iter()
            .filter_map(|status_entity| {
                active_status_query
                    .get(status_entity)
                    .ok()
                    .map(|existing_type| (existing_type.clone(), status_entity))
            })
            .collect();

        for (status_type, new_status_entities) in new_statuses_by_type {
            // Assume only one existing status per type, that is a given we are upholding here
            let total_status_count = statuses.iter().count();
            let unique_types = existing_status_by_type.len();
            if total_status_count != (unique_types + new_status_entities.length()) {
                panic!(
                    "Duplicate statuses detected! Had {} total statuses but only {} unique types",
                    total_status_count,
                    (unique_types + new_status_entities.length())
                );
            }

            // Keep only the longest lasting new status
            if let Some(longest_status_entity) = new_status_entities
                .iter()
                .max_by_key(|(_, lifespan)| lifespan.0.remaining().as_millis())
                .map(|(e, _)| *e)
            {
                // Despawn all other new statuses of this type
                for (status_entity, _) in new_status_entities {
                    if status_entity != longest_status_entity {
                        debug!("Despawning other {:?} status", status_type);
                        commands.entity(status_entity).despawn();
                    }
                }
                // Despawn existing status of the type if it exists
                if let Some(&existing_status) = existing_status_by_type.get(&status_type) {
                    commands.entity(existing_status).despawn();
                }

                // Mark only the chosen status as applied
                commands.entity(longest_status_entity).insert(StatusApplied);

                info!("Applied {:?} status", status_type);
            }
        }
    }
}
