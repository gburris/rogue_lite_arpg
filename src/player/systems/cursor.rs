use bevy::color::palettes::basic::WHITE;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::player::Player;

pub fn draw_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    gizmos.circle_2d(point, 10., WHITE);
}

pub fn face_cursor_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    // Get the primary window
    if let Ok(window) = windows.get_single() {
        // Get the cursor position in screen space
        if let Some(cursor_position) = window.cursor_position() {
            let screen_center_x = window.width() / 2.0;

            // Update the player's transform to face the cursor
            for mut transform in query.iter_mut() {
                if cursor_position.x < screen_center_x {
                    // Cursor is on the left side of the screen
                    transform.scale.x = 1.0; // Flip sprite to face left
                } else {
                    // Cursor is on the right side of the screen
                    transform.scale.x = -1.0; // Face right
                }
            }
        }
    }
}
