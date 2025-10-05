use avian2d::prelude::RigidBody;
use bevy::prelude::*;

use rand::{Rng, rng};

use crate::{
    character::player::PlayerStats,
    combat::{Health, damage::DefeatedEvent},
    economy::{GoldDropEvent, Purse},
    items::{Item, Items, lootable::ItemDropEvent},
    prelude::*,
    utility::Lifespan,
};

use super::{Enemy, Experience};

pub fn on_enemy_defeated(
    trigger: On<DefeatedEvent>,
    mut commands: Commands,
    mut defeated_enemy_query: Query<
        (&Experience, &Transform, Option<&Items>, Option<&Purse>),
        With<Enemy>,
    >,
    player_query: Single<(&PlayerStats, &mut Player)>,
    item_query: Query<&Item>,
) {
    let mut rng = rng();

    if let Ok((experience_to_gain, transform, items, purse)) =
        defeated_enemy_query.get_mut(trigger.target())
    {
        let (player_stats, mut player) = player_query.into_inner();
        //Give EXP to the player
        player.current_experience += experience_to_gain.base_exp;

        if let Some(items) = items {
            for item_entity in items.iter() {
                // Enemies drop their items based on drop rate
                if let Ok(item_result) = item_query.get(item_entity) {
                    let roll = rng.random_range(0.0..1.0);
                    if roll > (1.0 - item_result.drop_rate) {
                        commands.trigger(ItemDropEvent {
                            entity: item_entity,
                        });
                    }
                }
            }
        }

        if let Some(purse) = purse {
            // Enemies drop their gold based on player luck
            if rng.random_range(0.0..1.0) < (0.1 + (player_stats.luck as f32 / 100.0)) {
                commands.trigger(GoldDropEvent {
                    drop_location: transform.translation.truncate(),
                    amount: purse.amount,
                });
            }
        }

        commands
            .entity(trigger.target())
            .insert((Lifespan::new(2.0), ActionState::Defeated))
            .remove::<(Health, RigidBody)>()
            .despawn_related::<Children>();
    }
}
