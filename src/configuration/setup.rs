use bevy::prelude::*;

#[cfg(not(feature = "dev"))]
use bevy::asset::AssetMetaCheck;

use crate::progression::components::GameProgress;

#[cfg(feature = "dev")]
use crate::configuration::debug::DebugPlugin;

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

        app.insert_resource(GameProgress::default());
    }
}
