use crate::components::{Enemy, Player};
use bevy::prelude::*;

// Import the map bounds resource
pub fn move_enemies_toward_player(
    //Param set so I can query transform twice in one go
    mut query: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<(&mut Enemy, &mut Transform)>,
    )>,
) {
    
    // First, get the player position
    if let Ok(player_position) = query.p0().get_single() {
        let player_transform = &player_position;
        let player_pos = player_transform.translation;

        // Then, update enemy positions
        for (mut enemy, mut transform) in query.p1().iter_mut() {
            // Calculate direction toward the player
            let direction = Vec2::new(
                player_pos.x - enemy.position.x,
                player_pos.y - enemy.position.y,
            )
            .normalize_or_zero();

            // Update enemy position
            enemy.position.x += direction.x * enemy.speed;
            enemy.position.y += direction.y * enemy.speed;

            // Update the transform to reflect the new position
            transform.translation = Vec3::new(enemy.position.x, enemy.position.y, 1.0);
        }
    }
}
