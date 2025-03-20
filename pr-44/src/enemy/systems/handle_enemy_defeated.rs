use avian2d::prelude::ColliderDisabled;
use bevy::prelude::*;

use rand::{thread_rng, Rng};

use crate::{
    ai::state::ActionState,
    combat::{damage::DefeatedEvent, invulnerable::Invulnerable},
    despawn::components::LiveDuration,
    econ::gold_drop::GoldDropEvent,
    enemy::{Enemy, Experience},
    items::{inventory::inventory::Inventory, Item, ItemDropEvent},
    player::{
        components::{Player, PlayerExperience},
        PlayerStats,
    },
};

pub fn on_enemy_defeated(
    trigger: Trigger<DefeatedEvent>,
    mut commands: Commands,
    defeated_enemy_query: Query<(&Experience, &Transform, Option<&Inventory>), With<Enemy>>,
    player_query: Single<(&PlayerStats, &mut PlayerExperience), With<Player>>,
    item_query: Query<&Item>,
) {
    let mut rng = thread_rng();

    if let Ok((experience_to_gain, transform, inventory)) =
        defeated_enemy_query.get(trigger.entity())
    {
        let (player_stats, mut experience) = player_query.into_inner();
        //Give EXP to the player
        experience.current += experience_to_gain.base_exp;

        if let Some(inventory) = inventory {
            for item_entity in inventory.items.iter() {
                // Enemies drop their items based on drop rate
                if let Ok(item_result) = item_query.get(*item_entity) {
                    let roll = rng.gen_range(0.0..1.0);
                    if roll > (1.0 - item_result.drop_rate) {
                        commands.trigger_targets(ItemDropEvent, *item_entity);
                    }
                }
            }

            // Enemies drop their gold based on player luck
            if rng.gen_range(0.0..1.0) < (0.1 + (player_stats.luck as f32 / 100.0)) {
                commands.trigger(GoldDropEvent {
                    drop_location: *transform,
                    amount: inventory.coins,
                });
            }
        }

        commands
            .entity(trigger.entity())
            .insert(LiveDuration::new(2.0))
            .insert(ActionState::Defeated)
            .insert(ColliderDisabled)
            .insert(Invulnerable::death());
    }
}
