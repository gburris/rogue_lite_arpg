mod assets;
mod physics;
mod schedule;
mod view;

#[cfg(not(feature = "dev"))]
use bevy::asset::AssetMetaCheck;
#[cfg(feature = "dev")]
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::EnhancedInputPlugin;
use bevy_lit::prelude::Lighting2dPlugin;
use bevy_tweening::TweeningPlugin;

pub mod prelude {
    pub use super::assets::{GameIcons, Shadows, SpriteAssets, SpriteSheetLayouts, TileAssets};
    pub use super::physics::GameCollisionLayer;
    pub use super::schedule::{
        AppState, InGameSystems, MainSystems, PauseState, PlayingState, Restart, SimulationSystems,
        VfxSystems,
    };
    #[cfg(feature = "dev")]
    pub use super::view::draw_camera_debug_gizmos;
    pub use super::view::{CHARACTER_FEET_POS_OFFSET, YSort, ZLayer, shadow};
}

pub(super) fn plugin(app: &mut App) {
    #[cfg(feature = "dev")]
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: Level::INFO,
                // update game dev project to info or another when you get tired of debug
                filter: "wgpu=error,baba_yaga=debug".to_string(),
                ..default()
            })
            .set(view::window_plugin())
            .set(ImagePlugin::default_nearest()),
    );

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
            .set(view::window_plugin())
            .set(ImagePlugin::default_nearest()),
    );

    // Setup and configuration
    app.add_plugins((
        assets::plugin,
        physics::plugin,
        schedule::plugin,
        view::plugin,
    ));

    // Third-party plugins
    app.add_plugins((EnhancedInputPlugin, Lighting2dPlugin, TweeningPlugin));
}
