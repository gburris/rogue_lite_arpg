use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    configuration::debug::DebugPlugin,
    labels::{
        sets::MainSet,
        states::{AppState, PausedState, PlayingState},
    },
    progression::components::GameProgress,
};

use super::{configuration_data::ConfigurationData, view};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(debug_assertions)]
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
            .init_resource::<ConfigurationData>()
            .init_resource::<GameProgress>()
            .insert_resource(Gravity::ZERO) // no gravity since this is top-down game
            // initialize states
            .init_state::<AppState>()
            .add_sub_state::<PausedState>()
            .add_sub_state::<PlayingState>()
            .add_systems(Startup, view::spawn_camera)
            // avian recommendeds ordering camera following logic in PostUpdate after transform prop
            .add_systems(
                PostUpdate,
                view::camera_follow_system.before(TransformSystem::TransformPropagate),
            )
            .add_systems(FixedUpdate, view::ysort_transforms.in_set(MainSet::InGame));
    }
}
