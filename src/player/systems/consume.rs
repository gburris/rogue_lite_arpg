use crate::{
    combat::attributes::Health,
    items::{Charges, ConsumableEffect, ConsumableType},
};
use bevy::prelude::*;

#[derive(Event)]
pub struct ConsumeEvent {
    pub item_entity: Entity,
}

pub fn handle_consume_event(
    consume_trigger: Trigger<ConsumeEvent>,
    mut consumable_query: Query<(&ConsumableEffect, &mut Charges)>, // Only the potion has Charges
    mut health_query: Query<&mut Health>,                           // Player only has Health
) {
    let item_entity = consume_trigger.item_entity;

    // Get the consumable effect and charges from the item
    if let Ok((consumable, mut item_charges)) = consumable_query.get_mut(item_entity) {
        if item_charges.current < 15 {
            warn!("Not enough charges to use health potion!");
            return;
        }

        // Get the player's health (player no longer has Charges)
        if let Ok(mut health) = health_query.get_mut(consume_trigger.entity()) {
            match &consumable.effect_type {
                ConsumableType::Heal(amount) => {
                    let previous_hp = health.hp;
                    health.hp = (health.hp + amount).min(health.max_hp);
                    let healed_amount = health.hp - previous_hp;
                    info!(
                        "Player healed by {:.2} points (HP: {:.2}/{:.2})",
                        healed_amount, health.hp, health.max_hp
                    );
                }
            }

            // Consume 15 charges from the potion
            item_charges.current -= 15;
            info!("Health potion charges remaining: {}", item_charges.current);
        }
    }
}
