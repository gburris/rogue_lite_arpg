use avian2d::prelude::*;
use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    log::{Level, LogPlugin},
    prelude::*,
};

use crate::player::systems::player_input;

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
        .add_plugins(PhysicsDebugPlugin::default())
        // Enable system ambiguity detection
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .add_systems(Update, handle_debug_input.after(player_input));
    }
}

fn handle_debug_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    if keyboard_input.just_pressed(KeyCode::Comma) {
        let config = config_store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
    }
}
