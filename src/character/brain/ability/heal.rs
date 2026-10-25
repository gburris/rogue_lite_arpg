use bevy::prelude::*;

use super::AbilityCapability;
use crate::{
    character::brain::{Ability, Action, ActionFinished, ActionIntent, Brain, Cooldown, Tuning},
    prelude::*,
};

/// Drinking carried potions to recover health. Opt-in at spawn; absent means
/// the entity never drinks. The brain decides when; this component is the
/// body's capability and its pacing between potions.
#[derive(Component, Clone, Copy, Debug)]
pub struct Heal {
    /// Spacing between potions.
    pub cooldown: f32,
}

impl Heal {
    /// Drinks at most one potion per `cooldown` seconds.
    pub fn new(cooldown: f32) -> Self {
        Self {
            cooldown,
        }
    }
}

impl AbilityCapability for Heal {
    fn ability(&self) -> Ability {
        Ability::Heal
    }

    fn tuning(&self) -> Tuning {
        Tuning {
            cooldown: self.cooldown,
            ..Tuning::default()
        }
    }
}

/// Drinks a carried potion straight from the inventory. The intent refreshes
/// at the decision rate but this runs every frame, so the cooldown is what
/// keeps one tick from draining the whole stack.
pub(super) fn apply_heal(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    drinkers: Query<(Entity, &ActionIntent, &Brain, Option<&Items>), With<Heal>>,
    consumables: Query<(), With<Consumable>>,
) {
    let now = time.elapsed_secs();
    for (entity, intent, brain, items) in &drinkers {
        if !intent.get(Action::Ability(Ability::Heal)) || !brain.is_ready(Cooldown::Ability(Ability::Heal), now) {
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
            commands.trigger(ActionFinished {
                entity,
                action: Some(Action::Ability(Ability::Heal)),
            });
        }
    }
}
