use avian2d::prelude::*;
use bevy::{prelude::*, window::WindowResolution};

use crate::{
    configuration::debug::DebugPlugin,
    labels::states::{AppState, PausedState, PlayingState},
    progression::components::GameProgress,
};

#[derive(Component)]
pub struct CursorCoordinates;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(all(debug_assertions))]
        app.add_plugins(DebugPlugin);

        #[cfg(not(debug_assertions))]
        app.add_plugins(
            DefaultPlugins
                .set(get_release_window_plugin())
                .set(ImagePlugin::default_nearest()),
        );

        app
            // setup avian physics (used for forces, collision, etc...)
            // length unit here represents "pixels per meter" and is a way to indicate the
            // scale of your world to the physics engine for performance optimizations
            // In this case, our tiles are currently 32 x 32 pixels so we set the scale accordingly
            .add_plugins(PhysicsPlugins::default().with_length_unit(32.0))
            .insert_resource(GameProgress::default())
            .insert_resource(Gravity::ZERO) // no gravity since this is top-down game
            // initialize states
            .init_state::<AppState>()
            .add_sub_state::<PausedState>()
            .add_sub_state::<PlayingState>()
            .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn get_release_window_plugin() -> WindowPlugin {
    #[cfg(target_arch = "wasm32")]
    {
        WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Baba Yaga"),
                fit_canvas_to_parent: true,
                ..Default::default()
            }),
            ..default()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Right click to cast Icebolt Left Click to Cast Fireball"),
                resolution: WindowResolution::new(1920.0, 1080.0),
                ..Default::default()
            }),
            ..default()
        }
    }
}
