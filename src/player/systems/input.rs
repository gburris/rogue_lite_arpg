use bevy::prelude::*;

use crate::{
    movement::components::IsMoving,
    npc::events::AttemptDialogueInput,
    player::{Player, PlayerMovementEvent},
};

//Component with an event tag called
//Pause Input evemt
//and bevy macros for component and event
#[derive(Event)]
pub struct PauseInputEvent;

pub fn player_input(
    mut commands: Commands,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>, // Access keyboard input
    mut event_writer: EventWriter<PlayerMovementEvent>, // Dispatch movement events
    mut is_moving_query: Query<&mut IsMoving, With<Player>>,
) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        commands.trigger(PauseInputEvent);
        return;
    }

    if keyboard_input.clear_just_pressed(KeyCode::Space) {
        commands.trigger(AttemptDialogueInput);
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
    } else {
        is_moving_query.single_mut().0 = false;
    }
}
