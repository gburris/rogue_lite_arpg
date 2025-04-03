use crate::{
    combat::health::AttemptHealingEvent,
    items::{inventory::inventory::Inventory, ConsumableEffect, ConsumableType},
};
use bevy::prelude::*;

#[derive(Event)]
pub struct ConsumeEvent {
    pub item_entity: Entity,
}

pub fn handle_consume_event(
    consume_trigger: Trigger<ConsumeEvent>,
    mut commands: Commands,
    consumable_query: Query<&ConsumableEffect>,
    mut to_heal_query: Query<&mut Inventory>,
) {
    let item_entity = consume_trigger.item_entity;

    if let Ok(consumable) = consumable_query.get(item_entity) {
        if let Ok(mut inventory) = to_heal_query.get_mut(consume_trigger.entity()) {
            match &consumable.effect_type {
                ConsumableType::Heal(amount) => {
                    commands.trigger_targets(
                        AttemptHealingEvent { amount: *amount },
                        consume_trigger.entity(),
                    );
                }
            }
            // Once we are here we know the item was consumed, so we remove it from inventory and despawn it
            inventory
                .remove_item(item_entity)
                .expect("Went to consume item and it was not in inventory!");
            commands.entity(item_entity).despawn_recursive();
        }
    }
}
