use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::aim::AimInput;
use super::interact::PlayerInteractionInput;
use super::movement::PlayerMovement;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<Player>();

    // Use Equipment Logic
    app.add_observer(on_use_mainhand_input)
        .add_observer(on_use_offhand_input)
        .add_observer(on_use_offhand_complete);

    // Pause Logic
    app.add_systems(
        OnEnter(PauseState::Running),
        set_controls(ContextActivity::<Player>::ACTIVE),
    )
    .add_systems(
        OnEnter(PauseState::Paused),
        set_controls(ContextActivity::<Player>::INACTIVE),
    )
    .add_systems(
        OnEnter(PlayingState::Death),
        set_controls(ContextActivity::<Player>::INACTIVE),
    );
}

const MOUSE_SENSITIVITY: f32 = 0.5;
const CONTROLLER_AIM_SENSITIVITY: f32 = 8.0;

/// The player's gameplay controls, added to the player.
pub(super) fn player_actions() -> impl Scene {
    bsn! {
        Actions<Player> [
            Action::<PlayerInteractionInput>::new()
            Bindings [
                Binding(KeyCode::Space)
                --
                Binding(GamepadButton::South)
            ]
            --
            Action::<PlayerMovement>::new()
            DeadZone
            Bindings [
                {preset::cardinal::wasd_keys()}
                --
                {preset::axial::left_stick()}
            ]
            --
            Action::<AimInput>::new()
            Bindings [
                Binding::mouse_motion()
                ~Negate::y()
                ~Scale::splat(MOUSE_SENSITIVITY)
                --
                {preset::axial::new(
                    (
                        GamepadAxis::RightStickX,
                        DeadZone { upper_threshold: 0.8, ..default() },
                        Scale::splat(CONTROLLER_AIM_SENSITIVITY),
                    ),
                    (
                        GamepadAxis::RightStickY,
                        DeadZone { upper_threshold: 0.8, ..default() },
                        Scale::splat(CONTROLLER_AIM_SENSITIVITY),
                    ),
                )}
            ]
            --
            Action::<UseMainhand>::new()
            Bindings [
                Binding(MouseButton::Left)
                --
                Binding(GamepadButton::RightTrigger)
                --
                Binding(GamepadButton::RightTrigger2)
            ]
            --
            Action::<UseOffhand>::new()
            Bindings [
                Binding(MouseButton::Right)
                --
                Binding(GamepadButton::LeftTrigger)
                --
                Binding(GamepadButton::LeftTrigger2)
            ]
        ]
    }
}

fn set_controls(
    activity: ContextActivity<Player>,
) -> impl Fn(Commands, Single<Entity, With<Player>>) {
    move |mut commands, player| {
        commands.entity(*player).insert(activity);
    }
}

#[derive(InputAction)]
#[action_output(bool)]
pub(crate) struct UseMainhand;

#[derive(InputAction)]
#[action_output(bool)]
pub(crate) struct UseOffhand;

fn on_use_mainhand_input(use_mainhand: On<Start<UseMainhand>>, mut equipment: EquipmentUsage) {
    // Discarded: failures are already queued as `EquipmentUseFailed` for the HUD.
    let _ = equipment.try_use(use_mainhand.context, EquipmentSlot::Mainhand);
}

fn on_use_offhand_input(use_offhand: On<Start<UseOffhand>>, mut equipment: EquipmentUsage) {
    // Discarded: failures are already queued as `EquipmentUseFailed` for the HUD.
    let _ = equipment.try_use(use_offhand.context, EquipmentSlot::Offhand);
}

fn on_use_offhand_complete(use_offhand: On<Complete<UseOffhand>>, mut equipment: EquipmentUsage) {
    equipment.stop_using(use_offhand.context, EquipmentSlot::Offhand);
}
