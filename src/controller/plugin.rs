use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    ai::SimpleMotion,
    configuration::plugins::AppSettings,
    items::equipment::EquipmentSlot,
    labels::states::PausedState,
    player::{Player, UseEquipmentInputEvent},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<Node>()
            .add_input_context::<Player>();
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

#[derive(Debug, Event)]
pub struct PauseEvent(pub Option<PausedState>);

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct UseEquipMain;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct UseEquipOffhand;

pub fn player_binding(mut trigger: Trigger<Binding<Player>>, settings: Res<AppSettings>) {
    trigger.bind::<Movement>().to(settings.input.movement);
    trigger.bind::<Interact>().to(settings.input.interact);
    trigger.bind::<PauseRequest>().to(settings.input.pushing_P);
    trigger
        .bind::<UseEquipMain>()
        .to(settings.input.use_equip.main_hand);
    trigger
        .bind::<UseEquipOffhand>()
        .to(settings.input.use_equip.main_hand);
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

pub fn on_use_equip_main(
    equip: Trigger<Fired<UseEquipMain>>,
    mut commands: Commands,
    player_movement_query: Single<Entity, With<Player>>,
) {
    debug!("UseEquip triggered: {:?}", equip.value);
    let player_entity = player_movement_query.into_inner();
    commands.trigger_targets(
        UseEquipmentInputEvent {
            slot: EquipmentSlot::Mainhand,
        },
        player_entity,
    );
}
pub fn on_use_equip_offhand(
    equip: Trigger<Fired<UseEquipOffhand>>,
    mut commands: Commands,
    player_movement_query: Single<Entity, With<Player>>,
) {
    debug!("UseEquip triggered: {:?}", equip.value);
    let player_entity = player_movement_query.into_inner();
    commands.trigger_targets(
        UseEquipmentInputEvent {
            slot: EquipmentSlot::Offhand,
        },
        player_entity,
    );
}

//UN-Pause logic, runs when App State is Paused
pub fn on_system_menu(_: Trigger<Started<PauseRequest>>, mut commands: Commands) {
    debug!("ui_inputs, enter");
    commands.trigger(PauseEvent(None));
}
