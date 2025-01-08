use bevy::prelude::*;

use crate::npc::events::AttemptDialogueInput;
use crate::player::Inventory;
use crate::{labels::states::GameState, player::PlayerMovementEvent};

use super::print_inventory;

pub fn player_input(
    mut commands: Commands,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>, // Access keyboard input
    mut event_writer: EventWriter<PlayerMovementEvent>, // Dispatch movement events
    mut game_state: ResMut<NextState<GameState>>,
    query_inventory: Query<&Inventory>,
) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        game_state.set(GameState::Paused);
        return;
    }

    if keyboard_input.clear_just_pressed(KeyCode::Space) {
        commands.trigger(AttemptDialogueInput);
        return;
    }

    if keyboard_input.just_pressed(KeyCode::KeyI) {
        print_inventory(query_inventory);
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
