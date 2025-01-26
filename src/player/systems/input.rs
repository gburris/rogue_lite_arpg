use bevy::prelude::*;

use crate::player::{
    AttemptInteractionInput, MainHandActivated, Player, PlayerMovementEvent, PlayerStoppedEvent,
};

use crate::player::movement::MovementDirection;

#[derive(Event)]
pub struct PauseInputEvent;

pub fn player_input(
    mut commands: Commands,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>, // Access keyboard input
    buttons: Res<ButtonInput<MouseButton>>,
    mut event_writer: EventWriter<PlayerMovementEvent>, // Dispatch movement events
    is_moving_query: Single<(&MovementDirection, Entity), With<Player>>,
) {
    let (movement_direction, player_entity) = is_moving_query.into_inner();

    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        commands.trigger(PauseInputEvent);
        return;
    }

    if keyboard_input.clear_just_pressed(KeyCode::Space) {
        commands.trigger(AttemptInteractionInput);
        return;
    }

    if buttons.pressed(MouseButton::Left) {
        commands.trigger_targets(MainHandActivated, player_entity);
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
    // If we just stopped, dispatch stopped event
    if direction.length() > 0.0 {
        event_writer.send(PlayerMovementEvent { direction });
    } else {
        if *movement_direction != MovementDirection::None {
            commands.trigger(PlayerStoppedEvent);
        }
    }
}
