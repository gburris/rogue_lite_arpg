mod assets;
#[cfg(feature = "dev")]
pub mod debug;
mod physics;
mod schedule;
mod view;

#[cfg(not(feature = "dev"))]
use bevy::asset::AssetMetaCheck;
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
    pub use super::view::{CHARACTER_FEET_POS_OFFSET, YSort, ZLayer, shadow};
}

pub(super) fn plugin(app: &mut App) {
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
