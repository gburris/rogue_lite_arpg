use crate::components::{Enemy, Player, Position};
use crate::resources::MapBounds;
use bevy::prelude::*;

// Import the map bounds resource
pub fn move_enemies_toward_player(
    player_query: Query<&Position, With<Player>>,
    mut enemy_query: Query<(&mut Enemy, &mut Transform)>,
) {
    println!("Moving enemy to player");
    if let Ok(player_position) = player_query.get_single() {
        for (mut enemy, mut transform) in enemy_query.iter_mut() {
            // Calculate direction toward the player
            let direction = Vec2::new(
                player_position.x - enemy.position.x,
                player_position.y - enemy.position.y,
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
