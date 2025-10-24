use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_behave::prelude::BehavePlugin;

#[cfg(not(feature = "dev"))]
use bevy::asset::AssetMetaCheck;

use crate::{prelude::*, progression::components::GameProgress};

#[cfg(feature = "dev")]
use crate::configuration::debug::DebugPlugin;

use super::view;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "dev")]
        app.add_plugins(DebugPlugin);

        #[cfg(not(feature = "dev"))]
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
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
            .add_plugins(BehavePlugin::default())
            .add_systems(Startup, view::spawn_camera)
            // avian recommendeds ordering camera following logic in PostUpdate after transform prop
            .add_systems(
                PostUpdate,
                view::camera_follow_system.before(TransformSystems::Propagate),
            )
            .add_systems(
                FixedUpdate,
                view::ysort_transforms.in_set(MainSystems::InGame),
            );
    }
}
