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
    items::{inventory::inventory::Inventory, Item, ItemToGroundEvent},
    player::components::{Player, PlayerExperience},
};
use rand::{thread_rng, Rng};

pub fn on_enemy_defeated(
    trigger: Trigger<DefeatedEvent>,
    mut commands: Commands,
    mut defeated_enemy_query: Query<(&Experience, &Transform, Option<&Inventory>), With<Enemy>>,
    mut player_query: Query<&mut PlayerExperience, With<Player>>,
    item_query: Query<&Item>,
) {
    let mut rng = thread_rng();

    if let Ok((experience_to_gain, transform, inventory)) =
        defeated_enemy_query.get_mut(trigger.entity())
    {
        // Handle experience gain
        if let Ok(mut experience) = player_query.get_single_mut() {
            experience.current += experience_to_gain.base_exp;
        }

        if let Some(inventory) = inventory {
            for item_entity in inventory.items.iter() {
                if let Ok(item_result) = item_query.get(*item_entity) {
                    let roll = rng.gen_range(0.0..1.0);
                    if roll > (1.0 - item_result.drop_rate) {
                        commands.trigger_targets(
                            ItemToGroundEvent {
                                origin_position: transform.translation,
                            },
                            *item_entity,
                        );
                    }
                }
            }
        }

        if rng.gen_range(0.0..1.0) < 0.1 {
            commands.trigger(GoldDropEvent {
                drop_location: *transform,
                amount: experience_to_gain.base_exp,
                //For now, just drop the amount of EXP an enemy gives
                //10% of the time
            });
        }

        commands
            .entity(trigger.entity())
            .insert(LiveDuration::new(2.0))
            .insert(ActionState::Defeated)
            .insert(ColliderDisabled)
            .insert(Invulnerable::death());
    }
}
