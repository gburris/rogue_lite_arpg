use crate::{
    combat::damage::components::Health,
    items::{ConsumableEffect, ConsumableType},
    player::Inventory,
    ui::pause_menu::button_interactions::ConsumeEvent,
};
use bevy::prelude::*;

pub fn handle_consume_event(
    trigger: Trigger<ConsumeEvent>,
    mut commands: Commands,
    mut source_query: Query<(Entity, &mut Inventory)>,
    consumable_query: Query<&ConsumableEffect>, // Query for the consumable effect
    mut health_query: Query<&mut Health>,       // Query for the player's health
) {
    let Ok((player_entity, mut inventory)) = source_query.get_single_mut() else {
        warn!("No player inventory found");
        return;
    };

    // Find and remove the item from inventory
    if let Some(slot) = inventory
        .items
        .iter()
        .find(|(_, &entity)| entity == trigger.item_entity)
        .map(|(&slot, _)| slot)
    {
        if let Ok(consumable) = consumable_query.get(trigger.item_entity) {
            // Apply the consumable's effect
            if let Ok(mut health) = health_query.get_mut(player_entity) {
                match &consumable.effect_type {
                    ConsumableType::Heal(amount) => {
                        let previous_hp = health.hp;
                        health.hp = (health.hp + amount).min(health.max_hp); // Ensure HP does not exceed max
                        let healed_amount = health.hp - previous_hp;
                        warn!(
                            "Player healed by {:.2} points (HP: {:.2}/{:.2})",
                            healed_amount, health.hp, health.max_hp
                        );
                    }
                }
            }
        }

        // Remove the item from inventory
        inventory.items.remove(&slot);

        // Despawn the consumed item entity
        commands.entity(trigger.item_entity).despawn();
    }
}
