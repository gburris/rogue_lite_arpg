use crate::{
    combat::attributes::Health,
    items::{inventory::inventory::Inventory, ConsumableEffect, ConsumableType},
    ui::pause_menu::button_interactions::ConsumeEvent,
};
use bevy::prelude::*;

pub fn handle_consume_event(
    consume_trigger: Trigger<ConsumeEvent>,
    mut commands: Commands,
    consumable_query: Query<&ConsumableEffect>,
    mut to_heal_query: Query<(&mut Health, &mut Inventory)>,
) {
    let item_entity = consume_trigger.item_entity;

    if let Ok(consumable) = consumable_query.get(item_entity) {
        // Apply the consumable's effect
        if let Ok((mut health, mut inventory)) = to_heal_query.get_mut(consume_trigger.entity()) {
            match &consumable.effect_type {
                ConsumableType::Heal(amount) => {
                    let previous_hp = health.hp;
                    health.hp = (health.hp + amount).min(health.max_hp); // Ensure HP does not exceed max
                    let healed_amount = health.hp - previous_hp;
                    warn!(
                        "Entity healed by {:.2} points (HP: {:.2}/{:.2})",
                        healed_amount, health.hp, health.max_hp
                    );
                }
            }

            // Once we are here we know the item was consumed, so we remove it from inventory and despawn it
            inventory
                .remove_item_by_value(item_entity)
                .expect("Went to consume item and it was not in inventory!");
            commands.entity(item_entity).despawn();
        }
    }
}
