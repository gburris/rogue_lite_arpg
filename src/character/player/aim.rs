use bevy::{color::palettes::basic::WHITE, prelude::*, window::PrimaryWindow};

use crate::{character::state::Aim, prelude::*};

pub fn update_player_aim(
    mut player_aim: Single<&mut Aim, With<Player>>,
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

    player_aim.position = cursor_pos_in_world;
}

pub fn draw_cursor(player_aim: Single<&Aim, With<Player>>, mut gizmos: Gizmos) {
    gizmos.circle_2d(player_aim.position, 10., WHITE);
}
