use bevy::prelude::*;

use crate::components::Player;

// System for player movement
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut player, mut transform) in query.iter_mut() {
        let mut direction = Vec2::ZERO;
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
        player.position.x += direction.x * player.speed;
        player.position.y += direction.y * player.speed;
        transform.translation = Vec3::new(player.position.x, player.position.y, 1.0);
    }
}
