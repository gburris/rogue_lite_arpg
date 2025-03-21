use bevy::prelude::*;

use crate::{
    items::equipment::EquipmentSlot,
    labels::states::PausedState,
    player::{
        interact::PlayerInteractionInput, Player, PlayerMovementEvent, PlayerStoppedEvent,
        StopUsingHoldableEquipmentInputEvent, UseEquipmentInputEvent,
    },
};

#[derive(Event)]
pub struct PauseInputEvent {
    pub paused_state: Option<PausedState>, //What pause state to default to
}

pub fn player_input(
    mut commands: Commands,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>, // Access keyboard input
    buttons: Res<ButtonInput<MouseButton>>,
    mut event_writer: EventWriter<PlayerMovementEvent>, // Dispatch movement events
    player_movement_query: Single<Entity, With<Player>>,
) {
    let player_entity = player_movement_query.into_inner();

    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        commands.trigger(PauseInputEvent {
            paused_state: Some(PausedState::MainMenu),
        });
        return;
    }

    if keyboard_input.clear_just_pressed(KeyCode::Space) {
        commands.trigger(PlayerInteractionInput);
        return;
    }

    if buttons.pressed(MouseButton::Left) {
        commands.trigger_targets(
            UseEquipmentInputEvent {
                slot: EquipmentSlot::Mainhand,
            },
            player_entity,
        );
    }

    if buttons.just_pressed(MouseButton::Right) {
        commands.trigger_targets(
            UseEquipmentInputEvent {
                slot: EquipmentSlot::Offhand,
            },
            player_entity,
        );
    }
    if buttons.just_released(MouseButton::Right) {
        commands.trigger_targets(
            StopUsingHoldableEquipmentInputEvent {
                slot: EquipmentSlot::Offhand,
            },
            player_entity,
        );
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

    if direction.length() > 0.0 {
        event_writer.send(PlayerMovementEvent { direction });
    } else {
        commands.trigger(PlayerStoppedEvent);
    }
}
