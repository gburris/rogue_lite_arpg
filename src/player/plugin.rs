use bevy::prelude::*;

use crate::{
    labels::{
        sets::InGameSet,
        states::{AppState, PlayingState},
    },
    map::systems::state::transition_to_create_hub,
    player::{systems::*, PlayerMovementEvent},
};

use super::{
    animation::animation_setup::set_starting_player_animation_and_sprite_sheet,
    interact::{on_interaction_zone_added, on_player_interaction_input},
    systems::death::finish_death_animation,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMovementEvent>()
            .add_systems(
                OnEnter(AppState::SpawnPlayer),
                (
                    spawn_player,
                    set_starting_player_animation_and_sprite_sheet,
                    transition_to_create_hub,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                finish_death_animation
                    .in_set(InGameSet::Vfx)
                    .run_if(in_state(PlayingState::Death)),
            )
            .add_systems(
                Update,
                player_input
                    .in_set(InGameSet::PlayerInput)
                    .run_if(in_state(PlayingState::Playing)),
            )
            .add_systems(
                Update,
                (
                    (
                        player_movement,
                        update_player_aim_position,
                        on_player_experience_change,
                        camera_debug_system,
                    )
                        .in_set(InGameSet::Simulation),
                    // avian recommended ordering for camera following logic
                    camera_follow_system
                        .in_set(InGameSet::Camera)
                        .before(TransformSystem::TransformPropagate),
                    (draw_cursor, animate_level_up).in_set(InGameSet::Vfx),
                ),
            )
            .add_observer(handle_consume_event)
            .add_observer(on_level_up)
            .add_observer(on_player_stopped)
            .add_observer(on_player_interaction_input)
            .add_observer(on_interaction_zone_added);
    }
}
