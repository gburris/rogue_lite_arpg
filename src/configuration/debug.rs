use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{GREEN, YELLOW},
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    log::{Level, LogPlugin},
    prelude::*,
};

#[cfg(not(target_arch = "wasm32"))]
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;

use crate::{labels::sets::InGameSystems, prelude::Vision};

use super::view;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: Level::INFO,
                    // update game dev project to info or another when you get tired of debug
                    filter: "wgpu=error,baba_yaga=debug".to_string(),
                    ..default()
                })
                .set(view::get_window_plugin())
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(PhysicsDebugPlugin)
        .insert_gizmo_config(
            PhysicsGizmos::default(),
            GizmoConfig {
                enabled: false,
                ..default()
            },
        )
        // Enable system ambiguity detection
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .add_systems(
            Update,
            (
                handle_debug_input
                    .in_set(InGameSystems::PlayerInput)
                    .ambiguous_with_all(),
                (view::camera_debug_system, debug_vision)
                    .in_set(InGameSystems::HudOverlay)
                    .run_if(resource_exists::<DebugRenderEnabled>),
            ),
        );

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(FpsOverlayPlugin::default());
    }
}

#[derive(Resource)]
pub struct DebugRenderEnabled;

fn handle_debug_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config_store: ResMut<GizmoConfigStore>,
    debug_enabled: Option<Res<DebugRenderEnabled>>,
) {
    if keyboard_input.just_pressed(KeyCode::Comma) {
        if debug_enabled.is_some() {
            commands.remove_resource::<DebugRenderEnabled>();
        } else {
            commands.insert_resource(DebugRenderEnabled);
        }
        let config = config_store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
    }
}

/// Draws debug gizmos for AI vision direction and cone angles.
pub fn debug_vision(mut gizmos: Gizmos, query: Query<(&Transform, &Vision)>) {
    for (transform, vision) in &query {
        let origin = transform.translation.xy();
        let forward = vision.aim_direction;

        gizmos.arrow_2d(origin, origin + forward * 64.0, GREEN);

        let left = forward.rotate(Vec2::from_angle(45f32.to_radians()));
        let right = forward.rotate(Vec2::from_angle(-45f32.to_radians()));

        gizmos.line_2d(origin, origin + left * 64.0, YELLOW);
        gizmos.line_2d(origin, origin + right * 64.0, YELLOW);
    }
}
