use bevy::{
    color::palettes::css::WHITE,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused},
};
use bevy_enhanced_input::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_cursor.in_set(InGameSystems::Vfx));

    app.add_observer(on_player_aim);

    app.add_systems(OnEnter(AppState::Playing), grab_cursor)
        .add_systems(OnExit(Pause(true)), grab_cursor)
        .add_systems(
            Update,
            grab_cursor_when_focused.in_set(InGameSystems::PlayerInput),
        );
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub(super) struct PlayerAim;

fn on_player_aim(
    player_aim: On<Fire<PlayerAim>>,
    player: Single<(&mut Player, &mut Vision, &Transform)>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (mut player, mut player_vision, transform) = player.into_inner();

    if window.focused {
        player.aim_position += player_aim.value;

        let (camera, camera_transform) = *camera;

        let Some(aim_position_on_screen) =
            camera.world_to_ndc(camera_transform, player.aim_position.extend(0.0))
        else {
            return;
        };

        // Clamp the NDC coordinates to [-1.0, 1.0] range
        let clamped_ndc = aim_position_on_screen
            .xy()
            .clamp(Vec2::splat(-1.0), Vec2::splat(1.0));

        // Only update the aim position if we need to clamp
        if clamped_ndc != aim_position_on_screen.xy() {
            // Convert the clamped NDC back to world coordinates
            if let Some(clamped_world_pos) =
                camera.ndc_to_world(camera_transform, clamped_ndc.extend(0.0))
            {
                player.aim_position = clamped_world_pos.xy();
            }
        }

        player_vision.aim_direction =
            (player.aim_position - transform.translation.xy()).normalize_or_zero();
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

fn draw_cursor(player: Single<&Player>, mut gizmos: Gizmos) {
    gizmos.circle_2d(player.aim_position, 10., WHITE);
}
