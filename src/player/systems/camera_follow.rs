use bevy::{app::AppExit, prelude::*, window::WindowCloseRequested};

use crate::{map::WorldSpaceConfig, player::components::Player};

pub fn camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<(&mut Transform, &Camera), With<Camera>>,
    world_config: Res<WorldSpaceConfig>,
    windows: Query<&Window>,
    mut app_exit_events: EventWriter<AppExit>,
    close_requested_events: Res<Events<WindowCloseRequested>>,
) {
    if !close_requested_events.is_empty() {
        app_exit_events.send(AppExit::Success);
        return;
    }

    if let (Ok(player_transform), Ok((mut camera_transform, _camera))) =
        (player_query.get_single(), camera_query.get_single_mut())
    {
        let window = windows.single();

        let half_width = window.width() * 0.5;
        let half_height = window.height() * 0.5;

        // Calculate world boundaries based on WorldSpaceConfig
        let world_min_x = world_config.world_origin.x
            - (world_config.map_size.x as f32 * world_config.tile_size.x) / 2.0;
        let world_max_x = world_config.world_origin.x
            + (world_config.map_size.x as f32 * world_config.tile_size.x) / 2.0;
        let world_min_y = world_config.world_origin.y
            - (world_config.map_size.y as f32 * world_config.tile_size.y) / 2.0;
        let world_max_y = world_config.world_origin.y
            + (world_config.map_size.y as f32 * world_config.tile_size.y) / 2.0;

        let x = player_transform
            .translation
            .x
            .clamp(world_min_x + half_width, world_max_x - half_width);
        let y = player_transform
            .translation
            .y
            .clamp(world_min_y + half_height, world_max_y - half_height);

        camera_transform.translation.x = x;
        camera_transform.translation.y = y;
    }
}
