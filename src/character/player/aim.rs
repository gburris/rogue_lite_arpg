use bevy::{color::palettes::basic::WHITE, prelude::*, window::PrimaryWindow};

use crate::{character::state::Vision, prelude::*};

pub fn update_player_aim_position(
    mut player_aim_pos: Single<&mut Vision, With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = *camera_query;

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Ok(cursor_pos_in_world) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    player_aim_pos.position = cursor_pos_in_world;
}

pub fn draw_cursor(player_aim_pos: Single<&Vision, With<Player>>, mut gizmos: Gizmos) {
    gizmos.circle_2d(player_aim_pos.position, 10., WHITE);
}
