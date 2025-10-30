mod assets;
#[cfg(feature = "dev")]
pub mod debug;
mod physics;
mod schedule;
mod view;

#[cfg(not(feature = "dev"))]
use bevy::asset::AssetMetaCheck;

use bevy::prelude::*;
use bevy_behave::prelude::BehavePlugin;
use bevy_ecs_tilemap::prelude::TilemapPlugin;
use bevy_enhanced_input::prelude::EnhancedInputPlugin;

pub mod prelude {
    pub use super::assets::*;
    pub use super::physics::*;
    pub use super::schedule::*;
    pub use super::view::*;
}

pub(super) fn plugin(app: &mut App) {
    // Setup and configuration
    app.add_plugins((
        assets::plugin,
        physics::plugin,
        schedule::plugin,
        view::plugin,
    ));

    #[cfg(feature = "dev")]
    app.add_plugins(debug::plugin);

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

    // Third-party plugins
    app.add_plugins((BehavePlugin::default(), EnhancedInputPlugin, TilemapPlugin));
}
