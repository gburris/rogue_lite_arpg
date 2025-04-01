use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    controller::plugin::{
        on_movement, on_movement_stop, on_system_menu, on_use_equip_main, on_use_equip_offhand,
        player_binding,
    },
    labels::{
        sets::InGameSet,
        states::{AppState, PlayingState},
    },
    map::systems::state::transition_to_create_hub,
    player::systems::*,
};

use super::{
    interact::{on_interaction_zone_added, on_player_interaction_input},
    systems::death::finish_death_animation,
    Player,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>()
            .add_observer(player_binding)
            .add_observer(on_movement)
            .add_observer(on_movement_stop)
            .add_observer(on_use_equip_main)
            .add_observer(on_use_equip_offhand)
            .add_observer(on_system_menu)
            .add_systems(
                OnEnter(AppState::SpawnPlayer),
                (spawn_player, transition_to_create_hub).chain(),
            )
            .add_systems(
                Update,
                finish_death_animation
                    .in_set(InGameSet::Vfx)
                    .run_if(in_state(PlayingState::Death)),
            )
            .add_systems(
                Update,
                (
                    (update_player_aim_position, on_player_experience_change)
                        .in_set(InGameSet::Simulation),
                    (draw_cursor, animate_level_up).in_set(InGameSet::Vfx),
                ),
            )
            .add_observer(handle_consume_event)
            .add_observer(on_level_up)
            .add_observer(on_player_interaction_input)
            .add_observer(on_interaction_zone_added);
    }
}
