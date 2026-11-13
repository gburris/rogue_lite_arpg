use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;

use super::stat_modifier::StatModifier;
use super::status::{StatusKind, StatusOf, Statuses};
use crate::prelude::*;

#[derive(Component, Clone)]
#[require(Disabled, Lifespan)]
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
        (&StatusKind, Option<&StatModifier>, &Lifespan),
        (Without<StatusOf>, Allow<Disabled>),
    >,
    affected_query: Query<&Statuses>,
    status_query: Query<(&StatusKind, Option<&StatModifier>, &Lifespan), With<StatusOf>>,
) {
    let target = apply_effects.target;
    let statuses = affected_query.get(target).ok();
    let mut seen_kinds = Vec::new();
    for &effect in &apply_effects.effects {
        let Ok((kind, modifier, lifespan)) = effect_query.get(effect) else {
            warn!("Entity {effect:?} missing StatusKind component, skipping it");
            continue;
        };
        if seen_kinds.contains(kind) {
            continue;
        }
        seen_kinds.push(*kind);

        let existing = statuses
            .into_iter()
            .flat_map(Statuses::iter)
            .find_map(|status| {
                let (existing_kind, existing_modifier, existing_lifespan) =
                    status_query.get(status).ok()?;
                (existing_kind == kind).then_some((status, existing_modifier, existing_lifespan))
            });
        if let Some((status, existing_modifier, existing_lifespan)) = existing {
            if let (Some(modifier), Some(existing_modifier)) = (modifier, existing_modifier)
                && !modifier.outranks(
                    lifespan.remaining_secs(),
                    *existing_modifier,
                    existing_lifespan.remaining_secs(),
                )
            {
                continue;
            }
            commands.entity(status).despawn();
        }

        debug!("Applying effect {effect:?} to {target:?}");
        commands
            .entity(effect)
            .clone_and_spawn()
            .remove::<(Disabled, EffectOf)>()
            .insert(StatusOf(target));
    }
}
