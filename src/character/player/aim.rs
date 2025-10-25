use bevy::{color::palettes::css::WHITE, prelude::*, window::PrimaryWindow};

use crate::prelude::*;

pub(super) fn update_player_aim(
    player: Single<(&mut Player, &mut Vision, &Transform)>,
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

    let (mut player, mut player_vision, transform) = player.into_inner();

    player_vision.aim_direction =
        (cursor_pos_in_world - transform.translation.xy()).normalize_or_zero();
    player.aim_position = cursor_pos_in_world;
}

pub(super) fn draw_cursor(player: Single<&Player>, mut gizmos: Gizmos) {
    gizmos.circle_2d(player.aim_position, 10., WHITE);
}
