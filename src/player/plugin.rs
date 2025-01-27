use bevy::prelude::*;

use crate::{
    labels::{sets::InGameSet, states::AppState},
    map::systems::state::transition_to_create_hub,
    player::{resources::PlayerSize, systems::*, PlayerMovementEvent},
};

use super::animation::{
    animation_setup::set_starting_player_animation_and_sprite_sheet, plugin::PlayerAnimationPlugin,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMovementEvent>()
            .add_plugins(PlayerAnimationPlugin)
            .add_systems(
                OnEnter(AppState::SpawnPlayer),
                (
                    player_setup,
                    set_starting_player_animation_and_sprite_sheet,
                    transition_to_create_hub,
                )
                    .chain(),
            )
            .add_systems(Update, player_input.in_set(InGameSet::PlayerInput))
            .add_systems(
                Update,
                (
                    (
                        player_movement,
                        update_player_aim_position,
                        draw_cursor,
                        enforce_map_bounds,
                        on_player_experience_change,
                        animate_level_up,
                        regenerate_mana,
                    )
                        .before(camera_follow_system),
                    camera_follow_system.before(TransformSystem::TransformPropagate), // avian recommended ordering for camera following logic
                )
                    .in_set(InGameSet::Simulation),
            )
            .add_observer(reset_player_position)
            .add_observer(handle_consume_event)
            .add_observer(on_level_up)
            .add_observer(on_player_stopped)
            .insert_resource(PlayerSize { x: 256.0, y: 256.0 });
    }
}
