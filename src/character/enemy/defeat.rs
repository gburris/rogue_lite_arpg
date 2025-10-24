use avian2d::prelude::RigidBody;
use bevy::prelude::*;

use rand::{Rng, rng};

use crate::{
    character::{Purse, player::PlayerStats},
    combat::{Health, damage::Defeated},
    items::{Item, Items, lootable::ItemDrop},
    prelude::*,
    utility::Lifespan,
};

use crate::prelude::GoldDrop;

use super::{Enemy, Experience};

pub(super) fn on_enemy_defeated(
    defeated: On<Defeated>,
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
        defeated_enemy_query.get_mut(defeated.entity)
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
                        commands.trigger(ItemDrop {
                            entity: item_entity,
                        });
                    }
                }
            }
        }

        if let Some(purse) = purse {
            // Enemies drop their gold based on player luck
            if rng.random_range(0.0..1.0) < (0.1 + (player_stats.luck as f32 / 100.0)) {
                commands.trigger(GoldDrop {
                    location: transform.translation.truncate(),
                    amount: purse.amount,
                });
            }
        }

        commands
            .entity(defeated.entity)
            .insert((Lifespan::new(2.0), ActionState::Defeated))
            .remove::<(Health, RigidBody)>()
            .despawn_related::<Children>();
    }
}
