use crate::{combat::health::AttemptHeal, configuration::assets::SpriteAssets};
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
pub struct ConsumeEvent {
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

pub fn on_consume_event(
    consume_trigger: On<ConsumeEvent>,
    mut commands: Commands,
    consumable_query: Query<&Consumable>,
) {
    let item_entity = consume_trigger.item_entity;

    if let Ok(consumable) = consumable_query.get(item_entity) {
        match &consumable.effect {
            ConsumableType::Heal(amount) => {
                commands.trigger(AttemptHeal {
                    entity: consume_trigger.target(),
                    amount: *amount,
                });
            }
        }
        commands.entity(item_entity).despawn();
    }
}
