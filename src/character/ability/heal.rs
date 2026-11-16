use bevy::prelude::*;

use super::{Ability, AbilityCooldown, AbilityOf, AbilityTookEffect};
use crate::prelude::*;

/// Drinking a carried potion to recover health.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(Ability::Heal, AbilityCooldown)]
pub struct Heal;

/// A heal ability that drinks the holder's first carried potion when it takes effect.
pub fn heal() -> impl Scene {
    bsn! {
        Heal
        on(on_heal_took_effect)
    }
}

/// The first consumable the holder carries, the potion a [`Heal`] drinks.
pub(super) fn find_carried_potion(
    items: Option<&Items>,
    consumables: &Query<(), With<Consumable>>,
) -> Option<Entity> {
    items
        .into_iter()
        .flat_map(Items::iter)
        .find(|item| consumables.contains(*item))
}

/// Drinks the holder's first carried potion, if it still carries one.
fn on_heal_took_effect(
    took_effect: On<AbilityTookEffect>,
    mut commands: Commands,
    heals: Query<&AbilityOf, With<Heal>>,
    holders: Query<Option<&Items>>,
    consumables: Query<(), With<Consumable>>,
) {
    let Ok(ability_of) = heals.get(took_effect.entity) else {
        return;
    };
    let items = holders.get(ability_of.holder).ok().flatten();
    if let Some(potion) = find_carried_potion(items, &consumables) {
        commands.trigger(Consume {
            entity: ability_of.holder,
            item: potion,
        });
    }
}
