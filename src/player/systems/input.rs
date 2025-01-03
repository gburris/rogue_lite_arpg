use bevy::prelude::*;

use crate::player::PlayerMovementEvent;

pub fn player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>, // Access keyboard input
    mut event_writer: EventWriter<PlayerMovementEvent>, // Dispatch events
) {
    let mut direction = Vec2::ZERO;

    // Check input for movement and update direction
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

    // If there is movement input, dispatch the movement event
    if direction.length() > 0.0 {
        event_writer.send(PlayerMovementEvent { direction });
    }
}
