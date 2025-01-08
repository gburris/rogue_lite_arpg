use bevy::{app::AppExit, prelude::*, window::WindowCloseRequested};

use crate::{map::resources::MapBounds, player::components::Player};

pub fn camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<(&mut Transform, &Camera), With<Camera>>,
    mapbounds: Res<MapBounds>,
    windows: Query<&Window>,
    mut app_exit_events: EventWriter<AppExit>,
    close_requested_events: Res<Events<WindowCloseRequested>>,
) {
    if !close_requested_events.is_empty() {
        app_exit_events.send(AppExit::Success);
        return; // Exit early from the system if close event detected
    }

    if let (Ok(player_transform), Ok((mut camera_transform, _camera))) =
        (player_query.get_single(), camera_query.get_single_mut())
    {
        let window = windows.single();

        // Calculate the viewable area
        let half_width: f32 = window.width() * 0.5;
        let half_height = window.height() * 0.5;

        // Clamp the camera position
        let x = player_transform.translation.x.clamp(
            mapbounds.min_x as f32 + half_width,
            mapbounds.max_x as f32 - half_width,
        );
        let y = player_transform.translation.y.clamp(
            mapbounds.min_y as f32 + half_height,
            mapbounds.max_y as f32 - half_height,
        );

        camera_transform.translation.x = x;
        camera_transform.translation.y = y;
    }
}
