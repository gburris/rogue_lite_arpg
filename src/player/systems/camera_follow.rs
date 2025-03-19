use bevy::{app::AppExit, prelude::*, window::WindowCloseRequested};

use crate::{
    ai::{state::AimPosition},
    player::components::Player,
};

const DECAY_RATE: f32 = 2.9957; // f32::ln(20.0);
const TARGET_BIAS: f32 = 0.45; // 0.5 is middle of the two positions between the player and the aim position
const CAMERA_DISTANCE_CONSTRAINT: f32 = 300.0; // The camera will not go further than this distance from the player

pub fn camera_follow_system(
    player_query: Query<(&Transform, &AimPosition), (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut app_exit_events: EventWriter<AppExit>,
    close_requested_events: Res<Events<WindowCloseRequested>>,
    time: Res<Time>,
) {
    if !close_requested_events.is_empty() {
        app_exit_events.send(AppExit::Success);
        return;
    }

    if let (Ok((player, aim)), Ok(mut camera)) =
        (player_query.get_single(), camera_query.get_single_mut())
    {
        let aim_pos = Vec3::new(aim.position.x, aim.position.y, camera.translation.z);
        let player_pos = player.translation.with_z(camera.translation.z);
        let target = player_pos.lerp(aim_pos, TARGET_BIAS);
        // constraint the camera to not go too far from the player
        let offset =
            (target - player_pos).clamp_length_max(CAMERA_DISTANCE_CONSTRAINT) + player_pos;
        camera
            .translation
            .smooth_nudge(&offset, DECAY_RATE, time.delta_secs());
    }
    //TODO: The camera really shouldn't just follow you out of the bounds 50/50, it should still have some clamping
    //behavior
}
