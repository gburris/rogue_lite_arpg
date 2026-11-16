use bevy::prelude::*;

use super::Ability;
use crate::prelude::*;

/// Drinking a carried potion to recover health.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(Ability::Heal)]
pub struct Heal;

/// A heal ability that drinks the holder's first carried potion when it takes effect.
pub fn heal() -> impl Scene {
    bsn! {
        Heal
        ~{RequiredItem(ItemKind::Potion)}
        on(on_heal_took_effect)
    }
}

/// Drinks the holder's first carried potion, if it still carries one.
fn on_heal_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    heals: Query<&RequiredItem, With<Heal>>,
    holders: Query<&Items>,
    carried_items: Query<&Item>,
) {
    let Ok(&required_item) = heals.get(took_effect.entity) else {
        return;
    };
    let items = holders.get(took_effect.holder).ok();
    if let Some(potion) = required_item.find_carried(items, &carried_items) {
        commands.trigger(Consume {
            entity: took_effect.holder,
            item: potion,
        });
    }
}
