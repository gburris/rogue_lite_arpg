use bevy::prelude::*;

use crate::{
    animation::animate_sprite,
    combat::spells::cast_spell_system,
    labels::{sets::GamePlaySet, states::GameState},
    player::{resources::PlayerSize, systems::*, PlayerMovementEvent},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMovementEvent>()
            .add_observer(handle_enemy_collision)
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
                    on_player_experience_change,
                    animate_level_up,
                )
                    .in_set(GamePlaySet::Simulation),
            )
            .add_observer(reset_player_position)
            .add_observer(on_level_up)
            .insert_resource(PlayerSize { x: 256.0, y: 256.0 });
    }
}
