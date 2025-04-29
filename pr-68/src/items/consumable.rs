use crate::{
    combat::health::AttemptHealingEvent, configuration::assets::SpriteAssets,
    items::inventory::inventory::Inventory,
};
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

pub fn spawn_health_potion(commands: &mut Commands, sprites: &SpriteAssets) -> Entity {
    commands
        .spawn((
            Name::new("Health Potion"),
            Item::new(40, ItemType::Potion),
            Consumable {
                effect: ConsumableType::Heal(50.0), // Heals 50 HP
            },
            Sprite::from_image(sprites.health_potion.clone()),
        ))
        .id()
}

pub fn on_consume_event(
    consume_trigger: Trigger<ConsumeEvent>,
    mut commands: Commands,
    consumable_query: Query<&Consumable>,
    mut to_heal_query: Query<&mut Inventory>,
) {
    let item_entity = consume_trigger.item_entity;

    if let Ok(consumable) = consumable_query.get(item_entity) {
        if let Ok(mut inventory) = to_heal_query.get_mut(consume_trigger.target()) {
            match &consumable.effect {
                ConsumableType::Heal(amount) => {
                    commands.trigger_targets(
                        AttemptHealingEvent { amount: *amount },
                        consume_trigger.target(),
                    );
                }
            }
            // Once we are here we know the item was consumed, so we remove it from inventory and despawn it
            inventory
                .remove_item(item_entity)
                .expect("Went to consume item and it was not in inventory!");
            commands.entity(item_entity).despawn();
        }
    }
}
