use bevy::prelude::*;

use crate::{
    animation::animate_sprite,
    labels::{sets::GamePlaySet, states::GameState},
    player::{
        camera_follow_system, enforce_map_bounds, player_input, reset_player_position,
        systems::{draw_cursor, face_cursor_system, player_movement, player_setup},
        PlayerMovementEvent,
    },
    spells::cast_spell_system,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMovementEvent>()
            .add_systems(OnEnter(GameState::SpawnPlayer), player_setup)
            .add_systems(Update, player_input.in_set(GamePlaySet::PlayerInput))
            .add_systems(
                Update,
                (
                    // avian recommended ordering for camera following logic
                    camera_follow_system.before(TransformSystem::TransformPropagate),
                    player_movement,
                    face_cursor_system,
                    draw_cursor,
                    enforce_map_bounds,
                    cast_spell_system,
                    animate_sprite,
                )
                    .in_set(GamePlaySet::Simulation),
            )
            .add_observer(reset_player_position);
    }
}
