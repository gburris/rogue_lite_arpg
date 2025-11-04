use bevy::{
    color::palettes::css::WHITE,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused},
};
use bevy_enhanced_input::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_player_aim);

    app.add_systems(OnEnter(AppState::Playing), grab_cursor)
        .add_systems(OnExit(Pause(true)), grab_cursor)
        .add_systems(
            Update,
            grab_cursor_when_focused.in_set(InGameSystems::PlayerInput),
        );
}

#[derive(Component)]
pub struct PlayerAim;

pub(super) fn player_aim(mut gizmo_assets: ResMut<Assets<GizmoAsset>>) -> impl Bundle {
    let mut gizmo = GizmoAsset::default();

    gizmo.circle_2d(Vec2::ZERO, 6., WHITE);

    (
        PlayerAim,
        Gizmo {
            handle: gizmo_assets.add(gizmo),
            line_config: GizmoLineConfig {
                width: 4.,
                ..default()
            },
            ..default()
        },
    )
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub(super) struct AimInput;

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
        let world_aim_pos = player_transform.translation.xy() + player_aim.translation.xy();

        // Apply screen-relative input
        let new_world_aim_pos = world_aim_pos + aim_input.value;

        // Convert back to player-relative, clamped to max distance
        let player_pos = player_transform.translation.xy();
        let direction = new_world_aim_pos - player_pos;
        let distance = direction.length();

        if distance > AIM_DISTANCE {
            player_aim.translation = (direction.normalize() * AIM_DISTANCE).extend(0.0);
        } else {
            player_aim.translation = direction.extend(0.0);
        }

        player_vision.aim_direction = player_aim.translation.xy().normalize_or_zero();
    }
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
