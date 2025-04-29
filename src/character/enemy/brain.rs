use bevy::prelude::*;

use crate::{character::state::Vision, combat::Health, prelude::*};

pub fn update_enemy_aim_position(
    mut enemy_aim_pos_query: Query<&mut Vision, With<Enemy>>,
    player_transform: Single<&Transform, With<Player>>,
) {
    for mut aim_position in enemy_aim_pos_query.iter_mut() {
        aim_position.position = player_transform.translation.truncate();
    }
}

pub fn is_player_in_sight(
    mut enemy_query: Query<(&Health, &Transform, &mut Vision), (With<Enemy>, Without<NPC>)>,
    player_transform: Single<&Transform, With<Player>>,
) {
    const VISION_DISTANCE: f32 = 150.0;

    enemy_query
        .par_iter_mut()
        .for_each(|(health, transform, mut vision)| {
            let distance_to_player = player_transform
                .translation
                .xy()
                .distance(transform.translation.xy());

            if distance_to_player <= VISION_DISTANCE || health.hp < health.max_hp {
                vision.has_target = true;
            } else {
                vision.has_target = false;
            }
        });
}
