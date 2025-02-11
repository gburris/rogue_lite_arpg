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
    items::{inventory::inventory::Inventory, ItemToGroundEvent},
    player::components::{Player, PlayerExperience},
};

pub fn on_enemy_defeated(
    trigger: Trigger<DefeatedEvent>,
    mut commands: Commands,
    mut defeated_enemy_query: Query<(&Experience, &Transform, Option<&Inventory>), With<Enemy>>,
    mut player_query: Query<&mut PlayerExperience, With<Player>>,
) {
    if let Ok((experience_to_gain, transform, inventory)) =
        defeated_enemy_query.get_mut(trigger.entity())
    {
        // Handle experience gain
        if let Ok(mut experience) = player_query.get_single_mut() {
            experience.current += experience_to_gain.base_exp;
        }

        // Drop inventory items
        if let Some(inventory) = inventory {
            for item in inventory.items.iter() {
                commands.trigger_targets(
                    ItemToGroundEvent {
                        origin_position: transform.translation,
                    },
                    *item,
                );
            }
        }

        commands.trigger(GoldDropEvent {
            drop_location: *transform,
            amount: experience_to_gain.base_exp,
        });

        commands
            .entity(trigger.entity())
            .insert(LiveDuration::new(2.0))
            .insert(ActionState::Defeated)
            .insert(ColliderDisabled)
            .insert(Invulnerable::death());
    }
}
