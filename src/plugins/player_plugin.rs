use crate::{
    labels::{sets::GamePlaySet, states::GameState},
    systems::{
        animate_sprite, camera_follow_system, cast_spell_system, draw_cursor, face_cursor_system,
        player_movement, player_setup,
    },
};
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SpawnPlayer), player_setup)
            .add_systems(
                Update,
                // avian recommended ordering for camera following logic
                camera_follow_system
                    .before(TransformSystem::TransformPropagate)
                    .in_set(GamePlaySet::Simulation),
            )
            .add_systems(
                Update,
                (
                    player_movement,
                    face_cursor_system,
                    draw_cursor,
                    cast_spell_system,
                    animate_sprite,
                )
                    .in_set(GamePlaySet::Simulation),
            );
    }
}
