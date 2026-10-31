use bevy::prelude::*;

use super::{Item, ItemKind};
use crate::prelude::*;

/// What an item does when consumed.
#[derive(Component)]
pub enum Consumable {
    /// Heals the consumer by this amount.
    Heal(f32),
}

#[derive(EntityEvent)]
pub struct Consume {
    pub entity: Entity,
    pub item: Entity,
}

/// A healing potion. Potions live in the inventory and are consumed directly
/// via `Consume` (the same path the player uses); they are never equipped.
pub fn health_potion(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Health Potion"),
        Item::new(40, ItemKind::Potion),
        Consumable::Heal(50.0),
        Sprite::from_image(sprites.health_potion.clone()),
    )
}

pub(super) fn on_consume(
    consume: On<Consume>,
    mut commands: Commands,
    consumable_query: Query<&Consumable>,
) {
    let item = consume.item;

    if let Ok(consumable) = consumable_query.get(item) {
        match consumable {
            Consumable::Heal(amount) => {
                commands.trigger(AttemptHeal {
                    entity: consume.entity,
                    amount: *amount,
                });
            }
        }
        commands.entity(item).despawn();
    }
}
