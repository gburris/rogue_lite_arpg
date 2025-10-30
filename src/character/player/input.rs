use bevy::prelude::*;

use crate::prelude::*;

use super::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        player_input
            .in_set(InGameSystems::PlayerInput)
            .run_if(in_state(PlayingState::Playing)),
    );
}

fn player_input(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    player_movement_query: Single<Entity, With<Player>>,
) {
    let player_entity = player_movement_query.into_inner();

    if buttons.pressed(MouseButton::Left) {
        commands.trigger(UseEquipmentInput {
            entity: player_entity,
            slot: EquipmentSlot::Mainhand,
        });
    }

    if buttons.just_pressed(MouseButton::Right) {
        commands.trigger(UseEquipmentInput {
            entity: player_entity,
            slot: EquipmentSlot::Offhand,
        });
    }
    if buttons.just_released(MouseButton::Right) {
        commands.trigger(StopUsingHoldableEquipmentInput {
            entity: player_entity,
            slot: EquipmentSlot::Offhand,
        });
        return;
    }
}
