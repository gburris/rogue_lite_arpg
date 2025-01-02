use crate::components::Speed;
use crate::player::Player;
use crate::resources::MapBounds;
use crate::resources::PlayerSize;
use bevy::prelude::*; // Import the map bounds resource

// System for player movement
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>, // Correct resource type for keyboard input
    mapbounds: Res<MapBounds>,                 // Access the map bounds
    mut query: Query<(&mut Player, &mut Transform, &Speed)>,
    playersize: Res<PlayerSize>,
) {
    for (mut _player, mut transform, speed) in query.iter_mut() {
        let mut direction = Vec2::ZERO;

        // Check input for movement
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // Update player position
        transform.translation.x += direction.x * speed.velocity;
        transform.translation.y += direction.y * speed.velocity;

        // Clamp the player position within the map bounds
        let clamp_x = transform.translation.x.clamp(
            mapbounds.min_x + playersize.x / 2.0,
            mapbounds.max_x - playersize.x / 2.0,
        );
        let clamp_y = transform.translation.y.clamp(
            mapbounds.min_y + playersize.y / 2.0,
            mapbounds.max_y - playersize.y / 2.0,
        );

        // Update the transform to reflect the clamped position
        transform.translation = Vec3::new(clamp_x, clamp_y, 1.0);
    }
}
