use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    ai::SimpleMotion,
    configuration::plugins::AppSettings,
    items::equipment::EquipmentSlot,
    labels::states::{AppState, PausedState},
    player::{interact::PlayerInteractionInput, Player, UseEquipmentInputEvent},
};

#[derive(Component)]
pub struct CurrentInputContext;
#[derive(Component)]
pub struct PlayerInputContext;
#[derive(Component)]
pub struct MenuInputContext;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<PlayerInputContext>()
            .add_input_context::<MenuInputContext>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn((CurrentInputContext, MenuInputContext));
            })
            .add_systems(
                OnEnter(AppState::SpawnPlayer),
                |mut commands: Commands, query: Single<Entity, With<CurrentInputContext>>| {
                    commands.entity(*query).insert(PlayerInputContext);
                },
            )
            // Player
            .add_observer(player_binding)
            .add_observer(on_movement)
            .add_observer(on_interact)
            .add_observer(on_movement_stop)
            .add_observer(on_use_equip_main)
            .add_observer(on_use_equip_offhand)
            // System
            .add_observer(system_binding)
            .add_observer(on_pause_request);
    }
}

// Player InputActions
#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Movement;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Interact;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct PauseRequest;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct UseEquipMain;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct UseEquipOffhand;

#[derive(Debug, Event)]
pub struct PauseInputEvent(pub Option<PausedState>);

pub fn player_binding(
    mut trigger: Trigger<Binding<PlayerInputContext>>,
    settings: Res<AppSettings>,
) {
    trigger.bind::<Movement>().to(settings.input.movement);
    trigger.bind::<Interact>().to(settings.input.interact);
    trigger
        .bind::<UseEquipMain>()
        .to(settings.input.use_equip.main_hand);
    trigger
        .bind::<UseEquipOffhand>()
        .to(settings.input.use_equip.off_hand);
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

pub fn on_interact(_: Trigger<Started<Interact>>, mut commands: Commands) {
    commands.trigger(PlayerInteractionInput);
}

pub fn on_use_equip_main(
    _: Trigger<Fired<UseEquipMain>>,
    mut commands: Commands,
    player_movement_query: Single<Entity, With<Player>>,
) {
    let player_entity = player_movement_query.into_inner();
    commands.trigger_targets(
        UseEquipmentInputEvent {
            slot: EquipmentSlot::Mainhand,
        },
        player_entity,
    );
}
pub fn on_use_equip_offhand(
    _: Trigger<Fired<UseEquipOffhand>>,
    mut commands: Commands,
    player_movement_query: Single<Entity, With<Player>>,
) {
    let player_entity = player_movement_query.into_inner();
    commands.trigger_targets(
        UseEquipmentInputEvent {
            slot: EquipmentSlot::Offhand,
        },
        player_entity,
    );
}

pub fn system_binding(mut trigger: Trigger<Binding<MenuInputContext>>, settings: Res<AppSettings>) {
    trigger
        .bind::<PauseRequest>()
        .to(settings.input.pause_request);
}

//UN-Pause logic, runs when App State is Paused
pub fn on_pause_request(
    _: Trigger<Started<PauseRequest>>,
    _: Res<State<AppState>>,
    mut commands: Commands,
) {
    debug!("ui_inputs, enter");
    commands.trigger(PauseInputEvent(Some(PausedState::MainMenu)));
}
