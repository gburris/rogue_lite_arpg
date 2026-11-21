use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused};
use bevy_enhanced_input::prelude::*;

use super::interact::PlayerInteractionInput;
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

    app.add_observer(on_player_aim);

    app.add_systems(OnEnter(AppState::Playing), grab_cursor)
        .add_systems(OnExit(PauseState::Paused), grab_cursor)
        .add_systems(
            Update,
            grab_cursor_when_focused.in_set(InGameSystems::PlayerInput),
        );

    app.add_observer(on_player_stopped)
        .add_observer(on_player_movement);
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

const MAINHAND: ActionKey = ActionKey::Equipped(EquipmentSlot::Mainhand);
const OFFHAND: ActionKey = ActionKey::Equipped(EquipmentSlot::Offhand);

fn on_use_mainhand_input(use_mainhand: On<Start<UseMainhand>>, mut actions: ActionCommands) {
    // Discarded: failures are already queued as `ActionFailed` for the HUD.
    let _ = actions.try_use(use_mainhand.context, MAINHAND);
}

fn on_use_offhand_input(use_offhand: On<Start<UseOffhand>>, mut actions: ActionCommands) {
    // Discarded: failures are already queued as `ActionFailed` for the HUD.
    let _ = actions.try_use(use_offhand.context, OFFHAND);
}

fn on_use_offhand_complete(use_offhand: On<Complete<UseOffhand>>, mut actions: ActionCommands) {
    actions.stop_sustained_use(use_offhand.context, OFFHAND);
}

#[derive(Component, Default, Clone)]
pub struct PlayerAim;

/// The reticle the player aims with, a white ring offset from the player.
pub(super) fn player_aim() -> impl Scene {
    let mut reticle = GizmoAsset::default();
    reticle.circle_2d(Vec2::ZERO, 6., WHITE);

    bsn! {
        PlayerAim
        Gizmo {
            handle: asset_value(reticle),
            line_config: GizmoLineConfig { width: 4.0 },
        }
    }
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct AimInput;

const AIM_DISTANCE: f32 = 120.0;

fn on_player_aim(
    aim_input: On<Fire<AimInput>>,
    mut player_vision: Single<&mut Vision, With<Player>>,
    mut player_aim: Single<&mut Transform, With<PlayerAim>>,
    player_transform: Single<&Transform, (With<Player>, Without<PlayerAim>)>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if window.focused {
        // Convert player-relative aim to screen-relative world position
        let world_aim_position = player_transform.translation.xy() + player_aim.translation.xy();

        // Apply screen-relative input
        let new_world_aim_position = world_aim_position + aim_input.value;

        let offset = new_world_aim_position - player_transform.translation.xy();
        place_player_aim(offset, &mut player_vision, &mut player_aim);
    }
}

/// Moves the reticle to `offset` from the player, at most [`AIM_DISTANCE`] away, and points the
/// player's vision at it.
pub(crate) fn place_player_aim(offset: Vec2, vision: &mut Vision, aim_transform: &mut Transform) {
    let offset = offset.clamp_length_max(AIM_DISTANCE);

    aim_transform.translation = offset.extend(0.0);
    vision.aim_distance = offset.length();
    vision.aim_direction = offset.normalize_or_zero();
}

fn grab_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
}

fn grab_cursor_when_focused(
    mut window_focused_events: MessageReader<WindowFocused>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for window_focus in window_focused_events.read() {
        if window_focus.focused {
            cursor.visible = false;
            cursor.grab_mode = CursorGrabMode::Locked;
        }
    }
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub(crate) struct PlayerMovement;

fn on_player_movement(
    movement: On<Fire<PlayerMovement>>,
    mut player_kinematics: Single<&mut Kinematics, With<Player>>,
) {
    player_kinematics.move_in_direction(movement.value, 1.0);
}

fn on_player_stopped(
    _: On<Complete<PlayerMovement>>,
    mut player_kinematics: Single<&mut Kinematics, (With<Player>, Without<Enemy>, Without<Npc>)>,
) {
    player_kinematics.stop_moving();
}
