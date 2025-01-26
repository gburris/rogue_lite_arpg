use bevy::prelude::*;

use crate::{
    labels::{sets::InGameSet, states::AppState},
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
                    finish_player_setup,
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
                        tick_equippable_use_rate,
                        regenerate_mana,
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
            .add_observer(on_level_up)
            .add_observer(on_player_stopped)
            .insert_resource(PlayerSize { x: 256.0, y: 256.0 });
    }
}
