use bevy::{
    color::palettes::{basic::RED, css::BLUE, tailwind::PURPLE_700},
    prelude::*,
    render::camera::ScalingMode,
    window::WindowResolution,
};
use bevy_ecs_tilemap::map::{TilemapSize, TilemapTileSize};

use crate::{
    ai::state::AimPosition,
    map::components::{MapLayout, WorldSpaceConfig},
    player::components::Player,
};

#[derive(Component)]
pub struct YSort {
    z_layer: ZLayer,
}

impl Default for YSort {
    fn default() -> Self {
        Self::GROUNDED
    }
}

impl YSort {
    const GROUNDED: Self = YSort {
        z_layer: ZLayer::OnGround,
    };
}

pub fn ysort_transforms(
    mut transform_query: Query<(&mut Transform, &YSort)>,
    world_space_config: Res<WorldSpaceConfig>,
    map_layout: Res<MapLayout>,
) {
    for (mut transform, ysort) in transform_query.iter_mut() {
        let relative_height_on_map =
            transform.translation.y / (map_layout.size.y as f32 * world_space_config.tile_size.y);

        transform.translation.z = ysort.z_layer.z() - relative_height_on_map;
    }
}

pub enum ZLayer {
    Ground,
    OnGround,
    InAir,

    BehindSprite,
    AboveSprite,
}

impl ZLayer {
    pub fn z(&self) -> f32 {
        match self {
            ZLayer::Ground => 0.0,
            ZLayer::OnGround => 5.0,
            ZLayer::InAir => 10.0,

            // Z layer is additive in parent/child hierarchies
            // Parent 1 + child entity weapon of 0.1 = 1.1
            // These are the relative z layers
            ZLayer::BehindSprite => -0.5,
            ZLayer::AboveSprite => 0.5,
        }
    }
}

pub fn get_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: String::from("Baba Yaga"),
            fit_canvas_to_parent: cfg!(target_arch = "wasm32"),
            resolution: if cfg!(target_arch = "wasm32") {
                Default::default() // No resolution for wasm32
            } else {
                WindowResolution::new(1920.0, 1080.0) // Set resolution for non-WASM
            },
            ..default()
        }),
        ..default()
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 960.0,
                height: 540.0,
            },
            ..OrthographicProjection::default_2d()
        },
    ));
}

const DECAY_RATE: f32 = 2.9957; // f32::ln(20.0);
const TARGET_BIAS: f32 = 0.35; // 0.5 is middle of the two positions between the player and the aim position
const CAMERA_DISTANCE_CONSTRAINT: f32 = 120.0; // The camera will not go further than this distance from the player

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
