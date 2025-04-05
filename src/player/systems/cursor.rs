use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::ai::state::AimPosition;
use crate::player::Player;

pub fn update_player_aim_position(
    mut player_aim_pos: Single<&mut AimPosition, With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;
    // Calculate a world position based on the cursor's position.
    let cursor_pos = window
        .cursor_position()
        .and_then(|curp| camera.viewport_to_world_2d(camera_transform, curp).ok());
    if let Some(cursor_pos) = cursor_pos {
        player_aim_pos.position = cursor_pos;
        gizmos.circle_2d(player_aim_pos.position, 10., WHITE);
    };
}
