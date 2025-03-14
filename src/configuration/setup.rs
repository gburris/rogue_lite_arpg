use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    configuration::debug::DebugPlugin,
    labels::states::{AppState, PausedState, PlayingState},
    progression::components::GameProgress,
};

use super::view;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(all(debug_assertions))]
        app.add_plugins(DebugPlugin);

        #[cfg(not(debug_assertions))]
        app.add_plugins(
            DefaultPlugins
                .set(view::get_window_plugin())
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
            .add_systems(Startup, view::spawn_camera);
    }
}
