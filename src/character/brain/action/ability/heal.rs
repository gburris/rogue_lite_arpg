use bevy::prelude::*;

use crate::character::brain::Capability;
use crate::{
    character::brain::{Ability, Action, Brain, CapabilityProfile},
    prelude::*,
};

/// Drinking carried potions to recover health, paced by `cooldown`.
#[derive(Component, Clone, Copy, Debug)]
pub struct Heal {
    pub cooldown: f32,
}

impl Heal {
    /// Drinks at most one potion per `cooldown` seconds.
    pub fn new(cooldown: f32) -> Self {
        Self { cooldown }
    }
}

impl Capability for Heal {
    type Key = Ability;

    fn key(&self) -> Ability {
        Ability::Heal
    }

    fn capability_profile(&self) -> CapabilityProfile {
        CapabilityProfile {
            cooldown: self.cooldown,
            ..CapabilityProfile::default()
        }
    }
}

/// The first consumable the entity carries, the one check both the decision
/// and `apply_heal` use for whether a potion can be drunk.
pub(in crate::character::brain) fn find_carried_potion(
    items: Option<&Items>,
    consumables: &Query<(), With<Consumable>>,
) -> Option<Entity> {
    items
        .into_iter()
        .flat_map(Items::iter)
        .find(|item| consumables.contains(*item))
}

/// Drinks a carried potion straight from the inventory. The action refreshes
/// at the decision rate but this runs every frame, so the cooldown is what
/// keeps one tick from draining the whole stack.
pub(super) fn apply_heal(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    drinkers: Query<(Entity, &Brain, Option<&Items>), With<Heal>>,
    consumables: Query<(), With<Consumable>>,
) {
    let now = time.elapsed_secs();
    for (entity, brain, items) in &drinkers {
        let Some(drinking) = brain.held_action(Action::Ability(Ability::Heal)) else {
            continue;
        };
        let potion = find_carried_potion(items, &consumables)
            .filter(|_| brain.is_cooldown_ready(Ability::Heal, now));
        let Some(potion) = potion else {
            drinking.report_not_started(&mut commands, entity);
            continue;
        };
        commands.trigger(Consume {
            entity,
            item_entity: potion,
        });
        drinking.report_completed(&mut commands, entity);
    }
}
