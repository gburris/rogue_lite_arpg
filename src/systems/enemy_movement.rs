use crate::components::{Enemy, Player, Speed};
use bevy::prelude::*;

use bevy::prelude::*;

pub fn move_enemies_toward_player(
    mut param_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<(&Speed, &mut Transform), With<Enemy>>,
    )>,
) {
    // Get the player's transform (read-only)
    if let Ok(player_transform) = param_set.p0().get_single() {
        let player_pos = player_transform.translation;
        // Iterate through enemies and update their transforms
        for (speed, mut enemy_transform) in param_set.p1().iter_mut() {
            // Calculate direction towards the player
            let direction = player_pos - enemy_transform.translation;
            let direction = direction.normalize_or_zero();

            // Update the enemy's position
            enemy_transform.translation += direction * speed.velocity;
        }
    }
}
