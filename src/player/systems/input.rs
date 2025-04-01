mod _old {
    use bevy::prelude::*;

    use crate::{
        items::equipment::EquipmentSlot,
        player::{Player, UseEquipmentInputEvent},
    };
    pub fn _player_input(
        mut commands: Commands,
        mut keyboard_input: ResMut<ButtonInput<KeyCode>>, // Access keyboard input
        buttons: Res<ButtonInput<MouseButton>>,
        player_movement_query: Single<Entity, With<Player>>,
    ) {
        let player_entity = player_movement_query.into_inner();

        if buttons.pressed(MouseButton::Right) {
            commands.trigger_targets(
                UseEquipmentInputEvent {
                    slot: EquipmentSlot::Offhand,
                },
                player_entity,
            );
        }

        let mut direction = Vec2::ZERO;

        if buttons.pressed(MouseButton::Left) {
            commands.trigger_targets(
                UseEquipmentInputEvent {
                    slot: EquipmentSlot::Mainhand,
                },
                player_entity,
            );
        }

        if buttons.pressed(MouseButton::Right) {
            commands.trigger_targets(
                UseEquipmentInputEvent {
                    slot: EquipmentSlot::Offhand,
                },
                player_entity,
            );
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
        //
        // if direction.length() > 0.0 {
        //     event_writer.send(PlayerMovementEvent { direction });
        // } else {
        //     commands.trigger(PlayerStoppedEvent);
        // }
    }
}
