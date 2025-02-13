use avian2d::prelude::ColliderDisabled;
use bevy::prelude::*;

use crate::{
    combat::{
        components::ActionState,
        damage::{components::Invulnerable, events::DefeatedEvent},
    },
    despawn::components::LiveDuration,
    econ::components::GoldDropEvent,
    enemy::{Enemy, Experience},
    items::{
        inventory::inventory::Inventory, Charges, HealthPotion, Item, ItemDropEvent, RechargeEvent,
        RechargeOnKill,
    },
    player::{
        components::{Player, PlayerExperience},
        PlayerStats,
    },
};
use rand::{rngs::ThreadRng, thread_rng, Rng};

pub fn on_enemy_defeated(
    mut defeated_events: EventReader<DefeatedEvent>, // Use EventReader to read events
    mut commands: Commands,
    defeated_enemy_query: Query<(&Experience, &Transform, Option<&Inventory>), With<Enemy>>,
    mut player_query: Query<(&PlayerStats, &mut PlayerExperience, &Inventory), With<Player>>,
    health_potion_query: Query<&HealthPotion>,
    mut charges_query: Query<&mut Charges>,
    mut recharge_events: EventWriter<RechargeEvent>,
    item_query: Query<&Item>,
) {
    let mut rng = thread_rng();

    // Iterate over all DefeatedEvent events
    for event in defeated_events.read() {
        // Handle player data properly
        if let Ok((player_stats, mut experience, player_inventory)) = player_query.get_single_mut()
        {
            if let Ok((experience_to_gain, transform, enemy_inventory)) =
                defeated_enemy_query.get(event.entity)
            {
                // Handle experience gain
                handle_experience_gain(&mut experience, experience_to_gain.base_exp);

                // Recharge health potion (if needed)
                recharge_health_potion_on_kill(
                    &player_inventory,
                    &health_potion_query,
                    &mut charges_query,
                    &mut recharge_events,
                );

                // Handle item drops
                handle_item_drops(&mut commands, &mut rng, &enemy_inventory, &item_query);

                // Handle gold drops
                handle_gold_drops(
                    &mut commands,
                    &mut rng,
                    &player_stats,
                    &transform,
                    &enemy_inventory,
                );

                // Mark enemy as defeated
                mark_enemy_as_defeated(&mut commands, event.entity);
            }
        } else {
            warn!("Player not found or there was an error fetching player data.");
        }
    }
}

fn handle_experience_gain(experience: &mut PlayerExperience, exp_gain: u32) {
    experience.current += exp_gain;
    info!("Player gained {} experience!", exp_gain);
}

fn recharge_health_potion_on_kill(
    player_inventory: &Inventory,
    item_query: &Query<&HealthPotion>, // Corrected to Query<&HealthPotion>
    charges_query: &mut Query<&mut Charges>, // Corrected to &mut Query<&mut Charges>
    recharge_events: &mut EventWriter<RechargeEvent>,
) {
    for &item_entity in player_inventory.items.iter() {
        // Check if the item is a health potion
        if item_query.get(item_entity).is_ok() {
            // Recharge the health potion
            if let Ok(mut charges) = charges_query.get_mut(item_entity) {
                charges.current = (charges.current + 5).min(charges.max); // Recharge by 5
                info!("Health potion recharged! Charges: {}", charges.current);

                // Send a recharge event (if needed)
                recharge_events.send(RechargeEvent { item_entity });
                break; // Stop after recharging one health potion
            }
        }
    }
}

fn handle_item_drops(
    commands: &mut Commands,
    rng: &mut ThreadRng,
    enemy_inventory: &Option<&Inventory>,
    item_query: &Query<&Item>,
) {
    if let Some(inventory) = enemy_inventory {
        for &item_entity in inventory.items.iter() {
            if let Ok(item) = item_query.get(item_entity) {
                let roll = rng.gen_range(0.0..1.0);
                if roll > (1.0 - item.drop_rate) {
                    commands.trigger_targets(ItemDropEvent, item_entity);
                }
            }
        }
    }
}

fn handle_gold_drops(
    commands: &mut Commands,
    rng: &mut ThreadRng,
    player_stats: &PlayerStats,
    transform: &Transform,
    enemy_inventory: &Option<&Inventory>,
) {
    if let Some(inventory) = enemy_inventory {
        if rng.gen_range(0.0..1.0) < (0.1 + (player_stats.luck as f32 / 100.0)) {
            commands.trigger(GoldDropEvent {
                drop_location: *transform,
                amount: inventory.coins,
            });
        }
    }
}

fn mark_enemy_as_defeated(commands: &mut Commands, enemy_entity: Entity) {
    commands
        .entity(enemy_entity)
        .insert(LiveDuration::new(2.0))
        .insert(ActionState::Defeated)
        .insert(ColliderDisabled)
        .insert(Invulnerable::death());
}

pub fn handle_recharge_event(
    mut recharge_events: EventReader<RechargeEvent>,
    mut charges_query: Query<&mut Charges, With<RechargeOnKill>>,
) {
    for event in recharge_events.read() {
        if let Ok(mut charges) = charges_query.get_mut(event.item_entity) {
            // Recharge 5 charges, but don't exceed the maximum
            charges.current = (charges.current + 5).min(charges.max);
            info!("Health potion recharged! Charges: {}", charges.current);
        }
    }
}
