use bevy::{
    camera::ScalingMode,
    color::palettes::{basic::RED, css::BLUE, tailwind::PURPLE_700},
    prelude::*,
    window::WindowResolution,
};

use crate::prelude::*;

use super::assets::Shadows;

pub const CHARACTER_FEET_POS_OFFSET: f32 = -24.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera);

    // avian recommendeds ordering camera following logic in PostUpdate after transform prop
    app.add_systems(
        PostUpdate,
        camera_follow_system.before(TransformSystems::Propagate),
    );

    app.add_systems(FixedUpdate, ysort_transforms.in_set(MainSystems::InGame));
}

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

fn ysort_transforms(
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
    SpriteForeground,
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
            ZLayer::SpriteForeground => 2.0,
        }
    }
}

pub(super) fn get_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: String::from("Baba Yaga"),
            fit_canvas_to_parent: cfg!(target_arch = "wasm32"),
            resizable: false,
            resolution: if cfg!(target_arch = "wasm32") {
                Default::default() // No resolution for wasm32
            } else {
                WindowResolution::new(1280, 720) // Set resolution for non-WASM
            },
            ..default()
        }),
        ..default()
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 640.0,
                height: 360.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

const DECAY_RATE: f32 = 2.3; // f32::ln(10.0);
const TARGET_BIAS: f32 = 0.6; // 0.5 is middle of the two positions between the player and the aim position
const CAMERA_DISTANCE_CONSTRAINT: f32 = 400.0; // The camera will not go further than this distance from the player

fn camera_follow_system(
    player_transform: Single<&Transform, (With<Player>, Without<PlayerAim>, Without<Camera>)>,
    player_aim: Single<&Transform, With<PlayerAim>>,
    mut camera: Single<&mut Transform, (With<Camera>, Without<PlayerAim>, Without<Player>)>,
    time: Res<Time>,
) {
    let z = camera.translation.z;
    let aim: Vec3 = player_aim.translation.with_z(z);
    let player_pos = player_transform.translation.with_z(z);
    let target = player_pos.lerp(aim + player_pos, TARGET_BIAS);

    // apply a distance constraint to the camera, this keeps it close to the player
    // restore z from camera
    let offset = (target - player_pos).clamp_length_max(CAMERA_DISTANCE_CONSTRAINT) + player_pos;

    camera
        .translation
        .smooth_nudge(&offset, DECAY_RATE, time.delta_secs());
}

pub(super) fn camera_debug_system(
    player: Single<&Transform, (With<Player>, Without<PlayerAim>, Without<Camera>)>,
    player_aim: Single<&Transform, With<PlayerAim>>,
    mut gizmos: Gizmos,
) {
    let player_pos = player.translation.xy();
    let target = player_pos.lerp(player_aim.translation.xy() + player_pos, TARGET_BIAS);

    // apply a distance constraint to the camera, this keeps it close to the player
    // restore z from camera
    let offset = (target - player_pos).clamp_length_max(CAMERA_DISTANCE_CONSTRAINT) + player_pos;

    gizmos.circle_2d(target, 5.0, RED).resolution(64);
    gizmos.circle_2d(offset.xy(), 10.0, BLUE).resolution(64);
    gizmos
        .circle_2d(player_pos, CAMERA_DISTANCE_CONSTRAINT, PURPLE_700)
        .resolution(64);
}

pub fn shadow(shadows: &Shadows, y_offset: f32) -> impl Bundle {
    (
        Mesh2d(shadows.character_shadow.handle.clone()),
        MeshMaterial2d(shadows.shadow_color.handle.clone()),
        Transform::from_xyz(0.0, y_offset, ZLayer::SpriteBackground.z()),
    )
}
