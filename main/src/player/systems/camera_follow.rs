use bevy::{app::AppExit, prelude::*, window::WindowCloseRequested};

use crate::player::components::Player;

pub fn camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<(&mut Transform, &Camera), With<Camera>>,
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
        let x = player_transform.translation.x;
        let y = player_transform.translation.y;
        camera_transform.translation.x = x;
        camera_transform.translation.y = y;
    }
    //TODO: The camera really shouldn't just follow you out of the bounds 50/50, it should still have some clamping
    //behavior
}
