use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;

use super::stat_modifier::StatModifier;
use super::status::{StatusKind, StatusOf, Statuses};
use crate::prelude::*;

#[derive(Component, Clone)]
#[require(Disabled)]
#[relationship(relationship_target = Effects)]
pub struct EffectOf(Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = EffectOf, linked_spawn)]
pub struct Effects(Vec<Entity>);

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

/// Applies each effect to the target as a status, one per [`StatusKind`] with the first in the
/// list winning. A new status replaces the target's existing one of its kind, except that an
/// existing [`StatModifier`] stays when the new one does not [outrank](StatModifier::outranks) it.
pub(super) fn on_apply_effects(
    apply_effects: On<ApplyEffects>,
    mut commands: Commands,
    effect_query: Query<
        (&StatusKind, Option<&StatModifier>, Option<&Lifespan>),
        (Without<StatusOf>, Allow<Disabled>),
    >,
    affected_query: Query<&Statuses>,
    status_query: Query<(&StatusKind, Option<&StatModifier>, &Lifespan), With<StatusOf>>,
) {
    let mut new_effects: Vec<(StatusKind, Entity)> = Vec::new();
    for &effect in &apply_effects.effects {
        let Ok((kind, ..)) = effect_query.get(effect) else {
            warn!("Entity {effect:?} missing StatusKind component, skipping it");
            continue;
        };
        if new_effects.iter().all(|(new_kind, _)| new_kind != kind) {
            new_effects.push((*kind, effect));
        }
    }

    if let Ok(statuses) = affected_query.get(apply_effects.target) {
        for status in statuses.iter() {
            let Ok((kind, existing_modifier, existing_lifespan)) = status_query.get(status) else {
                continue;
            };
            let Some(index) = new_effects
                .iter()
                .position(|(new_kind, _)| new_kind == kind)
            else {
                continue;
            };
            let (_, effect) = new_effects[index];
            let keeps_existing = existing_modifier.is_some_and(|existing_modifier| {
                let Ok((_, Some(modifier), lifespan)) = effect_query.get(effect) else {
                    return false;
                };
                // A template without a `Lifespan` gets the default one `StatusOf` requires.
                let remaining_secs = lifespan.cloned().unwrap_or_default().remaining_secs();
                !modifier.outranks(
                    remaining_secs,
                    *existing_modifier,
                    existing_lifespan.remaining_secs(),
                )
            });
            if keeps_existing {
                new_effects.remove(index);
            } else {
                commands.entity(status).despawn();
            }
        }
    }

    debug!("Applying effects: {new_effects:?}");
    for (_, effect) in new_effects {
        commands
            .entity(effect)
            .clone_and_spawn()
            .remove::<(Disabled, EffectOf)>()
            .insert(StatusOf(apply_effects.target));
    }
}
