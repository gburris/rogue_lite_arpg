use bevy::prelude::*;

use super::AbilityCapability;
use crate::{
    character::brain::{Ability, AbilityIntent, AbilityTuning},
    prelude::*,
};

/// Drinking carried potions to recover health. Opt-in at spawn; absent means
/// the entity never drinks. The brain decides when; this component is the
/// body's capability and its pacing between potions.
#[derive(Component, Clone, Copy, Debug)]
pub struct Heal {
    /// Seconds between potions.
    pub cooldown: f32,
    ready_at: f32,
}

impl Heal {
    /// Drinks at most one potion per `cooldown` seconds.
    pub fn new(cooldown: f32) -> Self {
        Self {
            cooldown,
            ready_at: 0.0,
        }
    }
}

impl AbilityCapability for Heal {
    fn ability(&self) -> Ability {
        Ability::Heal
    }

    fn tuning(&self) -> AbilityTuning {
        AbilityTuning::default()
    }
}

/// Drinks a carried potion straight from the inventory; skips the dead. The
/// intent refreshes at the decision rate but this runs every frame, so the
/// cooldown is what keeps one tick from draining the whole stack.
pub(super) fn apply_heal(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut drinkers: Query<(
        Entity,
        &AbilityIntent,
        &mut Heal,
        Option<&Items>,
        Option<&Health>,
    )>,
    consumables: Query<(), With<Consumable>>,
) {
    let now = time.elapsed_secs();
    for (entity, intent, mut heal, items, health) in &mut drinkers {
        if !intent.get(Ability::Heal)
            || now < heal.ready_at
            || health.is_some_and(|health| health.hp <= 0.0)
        {
            continue;
        }
        let potion = items
            .iter()
            .flat_map(|items| items.iter())
            .find(|item| consumables.contains(*item));
        if let Some(potion) = potion {
            commands.trigger(Consume {
                entity,
                item_entity: potion,
            });
            heal.ready_at = now + heal.cooldown;
        }
    }
}
