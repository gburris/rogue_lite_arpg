use bevy::camera::ScalingMode;
use bevy::color::palettes::tailwind::BLUE_300;
#[cfg(feature = "dev")]
use bevy::color::palettes::{basic::RED, css::BLUE, tailwind::PURPLE_700};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_lit::prelude::{AmbientLight2d, Lighting2dSettings};

use super::assets::Shadows;
use crate::prelude::*;

pub const CHARACTER_FEET_POS_OFFSET: f32 = -24.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera);
    app.init_resource::<IntegerScale>();
    app.add_systems(Update, fit_integer_scale);

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
    /// in some instances we don't want to `YSort` from Sprite anchor, but instead
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
    tile_to_world: Res<TileToWorld>,
) {
    for (mut transform, ysort) in &mut transform_query {
        let relative_height_on_map = (transform.translation.y + ysort.height_offset)
            / (tile_to_world.map_size.y as f32 * tile_to_world.tile_size.y);

        transform.translation.z = ysort.z - relative_height_on_map;
    }
}

#[derive(Clone, Copy, Debug)]
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
    pub fn z(self) -> f32 {
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

pub(super) fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: String::from("Baba Yaga"),
            fit_canvas_to_parent: cfg!(target_arch = "wasm32"),
            resizable: false,
            resolution: if cfg!(target_arch = "wasm32") {
                WindowResolution::default() // No resolution for wasm32
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
            // Initial view; fit_integer_scale snaps it to exact integer
            // scaling on the first frame (and every resize after that).
            scaling_mode: ScalingMode::Fixed {
                width: 1280.0,
                height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Lighting2dSettings {
            blur: 4,
            edge_intensity: 8.0,
            ..default()
        },
        AmbientLight2d {
            intensity: 0.5,
            color: Color::from(BLUE_300),
        },
    ));
}

/// Target vertical view size in world units. The shown height snaps to the
/// nearest value that maps physical window pixels onto whole art texels.
/// 720 shows ~40x22 tiles at 1x on a 720p window; smaller values zoom in.
const VIEW_HEIGHT: f32 = 720.0;

/// Screen pixels per world unit; one art texel spans this many screen pixels.
#[derive(Resource, Clone, Copy)]
struct IntegerScale {
    steps: u32,
    seen_width: u32,
    seen_height: u32,
}

impl Default for IntegerScale {
    fn default() -> Self {
        Self {
            steps: 2,
            seen_width: 0,
            seen_height: 0,
        }
    }
}

/// Picks the integer scale nearest the target view whenever the window size
/// changes, so 1px tile lines sample evenly at any window size or OS scaling.
fn fit_integer_scale(
    window: Single<&Window>,
    mut projection: Single<&mut Projection, With<Camera>>,
    mut scale: ResMut<IntegerScale>,
) {
    let physical_width = window.resolution.physical_width();
    let physical_height = window.resolution.physical_height();
    if (physical_width, physical_height) == (scale.seen_width, scale.seen_height) {
        return;
    }
    scale.seen_width = physical_width;
    scale.seen_height = physical_height;
    scale.steps = ((physical_height as f32 / VIEW_HEIGHT).round().max(1.0)) as u32;
    let shown_height = physical_height as f32 / scale.steps as f32;
    let aspect = physical_width as f32 / physical_height as f32;
    if let Projection::Orthographic(ortho) = &mut **projection {
        ortho.scaling_mode = ScalingMode::Fixed {
            width: shown_height * aspect,
            height: shown_height,
        };
    }
}

const DECAY_RATE: f32 = 2.3; // f32::ln(10.0);
const TARGET_BIAS: f32 = 0.35; // 0.5 is middle of the two positions between the player and the aim position
const CAMERA_DISTANCE_CONSTRAINT: f32 = 120.0; // The camera will not go further than this distance from the player

fn camera_follow_system(
    player_transform: Single<&Transform, (With<Player>, Without<PlayerAim>, Without<Camera>)>,
    player_aim: Single<&Transform, With<PlayerAim>>,
    mut camera: Single<&mut Transform, (With<Camera>, Without<PlayerAim>, Without<Player>)>,
    time: Res<Time>,
    scale: Res<IntegerScale>,
) {
    let z = camera.translation.z;
    let aim: Vec3 = player_aim.translation.with_z(z);
    let player_position = player_transform.translation.with_z(z);
    let target = player_position.lerp(aim + player_position, TARGET_BIAS);

    // apply a distance constraint to the camera, this keeps it close to the player
    // restore z from camera
    let offset =
        (target - player_position).clamp_length_max(CAMERA_DISTANCE_CONSTRAINT) + player_position;

    camera
        .translation
        .smooth_nudge(&offset, DECAY_RATE, time.delta_secs());
    // Snap to whole screen pixels so the tilemap never sits at a fractional
    // offset, which shimmers 1px lines while moving.
    let step = 1.0 / scale.steps as f32;
    camera.translation.x = (camera.translation.x / step).round() * step;
    camera.translation.y = (camera.translation.y / step).round() * step;
}

#[cfg(feature = "dev")]
pub(super) fn camera_debug_system(
    player: Single<&Transform, (With<Player>, Without<PlayerAim>, Without<Camera>)>,
    player_aim: Single<&Transform, With<PlayerAim>>,
    mut gizmos: Gizmos,
) {
    let player_position = player.translation.xy();
    let target = player_position.lerp(player_aim.translation.xy() + player_position, TARGET_BIAS);

    // apply a distance constraint to the camera, this keeps it close to the player
    // restore z from camera
    let offset =
        (target - player_position).clamp_length_max(CAMERA_DISTANCE_CONSTRAINT) + player_position;

    gizmos.circle_2d(target, 5.0, RED).resolution(64);
    gizmos.circle_2d(offset.xy(), 10.0, BLUE).resolution(64);
    gizmos
        .circle_2d(player_position, CAMERA_DISTANCE_CONSTRAINT, PURPLE_700)
        .resolution(64);
}

pub fn shadow(shadows: &Shadows, y_offset: f32) -> impl Bundle {
    (
        Mesh2d(shadows.character_shadow.handle.clone()),
        MeshMaterial2d(shadows.shadow_color.handle.clone()),
        Transform::from_xyz(0.0, y_offset, ZLayer::SpriteBackground.z()),
    )
}
