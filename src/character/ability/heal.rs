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
    carried_items: CarriedItems,
) {
    let Ok(&RequiredItem(kind)) = heals.get(took_effect.source) else {
        return;
    };
    if let Some(potion) = carried_items.first_of_kind(took_effect.holder, kind) {
        commands.trigger(Consume {
            entity: took_effect.holder,
            item: potion,
        });
    }
}
