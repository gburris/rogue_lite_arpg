use core::panic;

use bevy::prelude::*;
use bevy_enhanced_input::{
    events::{Completed, Fired},
    input_action::ActionOutput,
    prelude::*,
};

use crate::{
    ai::SimpleMotion,
    items::equipment::EquipmentSlot,
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
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>()
            .add_observer(player_binding)
            .add_observer(on_movement)
            .add_observer(on_movement_stop)
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

// Player InputActions
#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Movement;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Interact;

#[derive(Debug, InputAction)]
#[input_action(output = EquipmentSlot)]
pub struct UseEquip;

impl ActionOutput for EquipmentSlot {
    const DIM: ActionValueDim = ActionValueDim::Bool;

    fn as_output(value: ActionValue) -> Self {
        panic!("{value:?}");
    }
}

use crate::{configuration::plugins::AppSettings, player::Player};
pub fn player_binding(mut trigger: Trigger<Binding<Player>>, settings: Res<AppSettings>) {
    trigger.bind::<Movement>().to(settings.input.movement);
    trigger.bind::<Interact>().to(settings.input.interact);
    trigger.bind::<UseEquip>().to(settings.input.use_equip);
}

pub fn on_movement(
    trigger: Trigger<Fired<Movement>>,
    mut player_motion: Single<&mut SimpleMotion, With<Player>>,
) {
    player_motion.start_moving(trigger.value);
}

pub fn on_movement_stop(
    _: Trigger<Completed<Movement>>,
    mut player_motion: Single<&mut SimpleMotion, With<Player>>,
) {
    player_motion.stop_moving();
}

pub fn on_use_equip(equip: Trigger<Fired<UseEquip>>) {
    debug!("UseEquip triggered: {:?}", equip.value);
}
