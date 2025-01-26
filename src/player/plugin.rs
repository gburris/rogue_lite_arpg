use bevy::prelude::*;

use crate::{
    animation::animate_sprite,
    labels::{sets::InGameSet, states::AppState},
    player::{resources::PlayerSize, systems::*, PlayerMovementEvent},
};

use super::animation::{
    animate_player::update_player_animation, components::PlayerAnimationConfig,
    run_player_animation::run_player_animation,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMovementEvent>()
            .add_systems(OnEnter(AppState::SpawnPlayer), player_setup)
            .add_systems(Update, player_input.in_set(InGameSet::PlayerInput))
            .add_systems(
                Update,
                (
                    (
                        player_movement,
                        update_player_aim_position,
                        face_cursor_system,
                        draw_cursor,
                        enforce_map_bounds,
                        animate_sprite,
                        on_player_experience_change,
                        animate_level_up,
                        tick_equippable_use_rate,
                        regenerate_mana,
                        update_player_animation, //Change animation if animation component changes
                        run_player_animation,    //Process the frames of the animation
                    )
                        .before(camera_follow_system),
                    camera_follow_system.before(TransformSystem::TransformPropagate), // avian recommended ordering for camera following logic
                )
                    .in_set(InGameSet::Simulation),
            )
            .add_observer(reset_player_position)
            .add_observer(handle_try_equip_event)
            .add_observer(handle_equip_success_event)
            .add_observer(handle_try_unequip_event)
            .add_observer(handle_unequip_success_event)
            .add_observer(handle_consume_event)
            .add_observer(on_player_stopped)
            .insert_resource(PlayerAnimationConfig::default())
            .add_observer(on_level_up)
            .insert_resource(PlayerSize { x: 256.0, y: 256.0 });
    }
}
