use bevy::prelude::*;

use crate::{
    animation::animate_sprite,
    combat::spells::cast_spell_system,
    labels::{sets::InGameSet, states::AppState},
    player::{resources::PlayerSize, systems::*, PlayerMovementEvent},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMovementEvent>()
            .add_observer(handle_enemy_collision)
            .add_systems(OnEnter(AppState::SpawnPlayer), player_setup)
            .add_systems(Update, player_input.in_set(InGameSet::PlayerInput))
            .add_systems(
                Update,
                (
                    (
                        player_movement,
                        face_cursor_system,
                        draw_cursor,
                        enforce_map_bounds,
                        cast_spell_system,
                        animate_sprite,
                        on_player_experience_change,
                        animate_level_up,
                    )
                        .before(camera_follow_system),
                    camera_follow_system.before(TransformSystem::TransformPropagate), // avian recommended ordering for camera following logic
                )
                    .in_set(InGameSet::Simulation),
            )
            .add_observer(reset_player_position)
            .add_observer(handle_equip_event)
            .add_observer(handle_consume_event)
            .add_observer(on_level_up)
            .insert_resource(PlayerSize { x: 256.0, y: 256.0 });
    }
}
