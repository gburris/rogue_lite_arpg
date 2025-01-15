use avian2d::prelude::*;
use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
    window::WindowResolution,
};

use crate::labels::states::{AppState, InGameState, PausedState};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        // Configure main window
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from(
                            "Right click to cast Icebolt Left Click to Cast Fireball",
                        ),
                        resolution: WindowResolution::new(1920.0, 1080.0),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        // Enable system ambiguity detection
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        // setup avian physics (used for forces, collision, etc...)
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .insert_resource(Gravity::ZERO) // no gravity since this is top-down game
        // initialize states
        .init_state::<AppState>()
        .init_state::<InGameState>()
        .add_sub_state::<PausedState>()
        .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
