use bevy::color::palettes::css::BLUE;
use bevy::color::palettes::tailwind::PURPLE_700;
use bevy::color::palettes::{basic::RED, css::ORANGE};
use bevy::prelude::*;

use crate::{ai::state::AimPosition, player::components::Player};

const DECAY_RATE: f32 = 2.9957; // f32::ln(20.0);
const TARGET_BIAS: f32 = 0.45; // 0.5 is middle of the two positions between the player and the aim position
const CAMERA_DISTANCE_CONSTRAINT: f32 = 300.0; // The camera will not go further than this distance from the player

#[allow(clippy::type_complexity)]
pub fn camera_follow_system(
    pq: Query<(&Transform, &AimPosition), (With<Player>, Without<Camera>)>,
    mut cq: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
) {
    let (Ok((player, aim)), Ok(mut camera)) = (pq.get_single(), cq.get_single_mut()) else {
        return;
    };

    let z = camera.translation.z;
    let aim_pos = Vec3::new(aim.position.x, aim.position.y, z);
    let player_pos = player.translation.with_z(z);
    let target = player_pos.lerp(aim_pos, TARGET_BIAS);

    // apply a distance constraint to the camera, this keeps it close to the player
    // restore z from camera
    let offset = (target - player_pos).clamp_length_max(CAMERA_DISTANCE_CONSTRAINT) + player_pos;

    camera
        .translation
        .smooth_nudge(&offset, DECAY_RATE, time.delta_secs());
}

#[allow(clippy::type_complexity)]
pub fn camera_debug_system(
    pq: Query<(&Transform, &AimPosition), (With<Player>, Without<Camera>)>,
    mut gizmos: Gizmos,
) {
    let Ok((player, aim)) = pq.get_single() else {
        return;
    };

    let player_pos = player.translation.xy();
    let target = player_pos.lerp(aim.position, TARGET_BIAS);

    // apply a distance constraint to the camera, this keeps it close to the player
    // restore z from camera
    let offset = (target - player_pos).clamp_length_max(CAMERA_DISTANCE_CONSTRAINT) + player_pos;

    gizmos.circle_2d(target, 5.0, RED).resolution(64);
    gizmos.circle_2d(offset.xy(), 10.0, BLUE).resolution(64);
    gizmos
        .circle_2d(player_pos, CAMERA_DISTANCE_CONSTRAINT, PURPLE_700)
        .resolution(64);
}
