use crate::{combat::health::AttemptHeal, prelude::SpriteAssets};
use bevy::prelude::*;

use super::{Item, ItemType};

#[derive(Component)]
pub struct Consumable {
    pub effect: ConsumableType,
}

pub enum ConsumableType {
    Heal(f32), // Heal player for a specific amount
}

#[derive(EntityEvent)]
pub struct Consume {
    pub entity: Entity,
    pub item_entity: Entity,
}

pub fn health_potion(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Health Potion"),
        Item::new(40, ItemType::Potion),
        Consumable {
            effect: ConsumableType::Heal(50.0), // Heals 50 HP
        },
        Sprite::from_image(sprites.health_potion.clone()),
    )
}

pub(super) fn on_consume_event(
    consume: On<Consume>,
    mut commands: Commands,
    consumable_query: Query<&Consumable>,
) {
    let item_entity = consume.item_entity;

    if let Ok(consumable) = consumable_query.get(item_entity) {
        match &consumable.effect {
            ConsumableType::Heal(amount) => {
                commands.trigger(AttemptHeal {
                    entity: consume.entity,
                    amount: *amount,
                });
            }
        }
        commands.entity(item_entity).despawn();
    }
}
