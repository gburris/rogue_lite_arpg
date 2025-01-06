use bevy::prelude::*;

use crate::{labels::states::GameState, player::PlayerMovementEvent};

pub fn player_input(
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>, // Access keyboard input
    mut event_writer: EventWriter<PlayerMovementEvent>, // Dispatch movement events
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        game_state.set(GameState::Paused);
        return;
    }

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
