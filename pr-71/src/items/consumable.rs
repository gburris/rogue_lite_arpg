use crate::{combat::health::AttemptHealingEvent, configuration::assets::SpriteAssets};
use bevy::prelude::*;

use super::{Item, ItemType};

#[derive(Component)]
pub struct Consumable {
    pub effect: ConsumableType,
}

pub enum ConsumableType {
    Heal(f32), // Heal player for a specific amount
}

#[derive(Event)]
pub struct ConsumeEvent {
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
    consume_trigger: Trigger<ConsumeEvent>,
    mut commands: Commands,
    consumable_query: Query<&Consumable>,
) {
    let item_entity = consume_trigger.item_entity;

    if let Ok(consumable) = consumable_query.get(item_entity) {
        match &consumable.effect {
            ConsumableType::Heal(amount) => {
                commands.trigger_targets(
                    AttemptHealingEvent { amount: *amount },
                    consume_trigger.target(),
                );
            }
        }
        commands.entity(item_entity).despawn();
    }
}
