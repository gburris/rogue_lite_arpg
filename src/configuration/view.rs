use bevy::{
    color::palettes::{basic::RED, css::BLUE, tailwind::PURPLE_700},
    prelude::*,
    render::camera::ScalingMode,
    window::WindowResolution,
};

use crate::{
    ai::state::AimPosition,
    map::components::{MapLayout, WorldSpaceConfig},
    player::components::Player,
};

use super::{assets::Shadows, configuration_data::ConfigurationData};

#[derive(Component)]
pub struct YSort {
    /// z layer of sprite, only sprites on the same layer will be y-sorted correctly
    z: f32,
    /// in some instances we don't want to YSort from Sprite anchor, but instead
    /// from the feet or some other position on the sprite
    height_offset: f32,
}

impl Default for YSort {
    fn default() -> Self {
        Self::from_z(ZLayer::OnGround)
    }
}

impl YSort {
    pub fn from_z(z_layer: ZLayer) -> Self {
        Self {
            z: z_layer.z(),
            height_offset: 0.0,
        }
    }

    pub fn from_offset(height_offset: f32) -> Self {
        Self {
            height_offset,
            ..default()
        }
    }
}

pub fn ysort_transforms(
    mut transform_query: Query<(&mut Transform, &YSort)>,
    world_space_config: Res<WorldSpaceConfig>,
    map_layout: Res<MapLayout>,
) {
    for (mut transform, ysort) in transform_query.iter_mut() {
        let relative_height_on_map = (transform.translation.y + ysort.height_offset)
            / (map_layout.size.y as f32 * world_space_config.tile_size.y);

        transform.translation.z = ysort.z - relative_height_on_map;
    }
}

pub enum ZLayer {
    Ground,
    OnGround,
    InAir,

    SpriteBackground,
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
            ZLayer::SpriteBackground => -2.0,
            ZLayer::BehindSprite => -0.001,
            ZLayer::AboveSprite => 0.001,
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

pub fn camera_follow_system(
    pq: Query<(&Transform, &AimPosition), (With<Player>, Without<Camera>)>,
    mut cq: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
    mut gizmos: Gizmos,
    configuration_data: Res<ConfigurationData>,
) {
    let ConfigurationData {
        CAMERA_DISTANCE_CONSTRAINT,
        TARGET_BIAS,
        DECAY_RATE,
        ..
    } = *configuration_data;

    let (Ok((player, aim)), Ok(mut camera)) = (pq.get_single(), cq.get_single_mut()) else {
        return;
    };

    let z = camera.translation.z;
    let aim_pos = Vec3::new(aim.position.x, aim.position.y, z);
    let player_pos = player.translation.with_z(z);
    gizmos
        .circle_2d(player_pos.xy(), CAMERA_DISTANCE_CONSTRAINT, PURPLE_700)
        .resolution(64);

    let target = player_pos.lerp(aim_pos, TARGET_BIAS);
    gizmos.circle_2d(target.xy(), 5.0, RED).resolution(64);

    // apply a distance constraint to the camera, this keeps it close to the player
    // restore z from camera
    let offset = (target - player_pos).clamp_length_max(CAMERA_DISTANCE_CONSTRAINT) + player_pos;
    gizmos.circle_2d(offset.xy(), 10.0, BLUE).resolution(64);

    camera.translation.smooth_nudge(&offset, DECAY_RATE, time.delta_secs());
}

pub fn spawn_shadow(spawner: &mut ChildBuilder, shadows: &Shadows, y_offset: f32) {
    spawner.spawn((
        Mesh2d(shadows.character_shadow.handle.clone()),
        MeshMaterial2d(shadows.shadow_color.handle.clone()),
        Transform::from_xyz(0.0, y_offset, ZLayer::SpriteBackground.z()),
    ));
}
