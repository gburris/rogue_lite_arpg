use crate::components::Enemy;
use crate::resources::MapBounds;
use bevy::prelude::*; // Import the map bounds resource

// System for player movement
pub fn move_enemies(
    // Correct resource type for keyboard input
    mapbounds: Res<MapBounds>, // Access the map bounds
    mut query: Query<(&mut Enemy, &mut Transform)>,
) {
    for (mut enemy, mut transform) in query.iter_mut() {
        let mut direction = Vec2::ZERO;
        direction.y += 1.0;
        direction.x += 1.0;

        // Update player position
        enemy.position.x += direction.x * enemy.speed;
        enemy.position.y += direction.y * enemy.speed;

        // Update the transform to reflect the clamped position
        transform.translation = Vec3::new(enemy.position.x, enemy.position.y, 1.0);
    }
}
