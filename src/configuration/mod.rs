mod assets;
#[cfg(feature = "dev")]
pub mod debug;
mod physics;
mod schedule;
pub mod setup;
mod view;

use bevy_behave::prelude::BehavePlugin;
pub use physics::GameCollisionLayer;
pub use view::CHARACTER_FEET_POS_OFFSET;
pub use view::YSort;
pub use view::ZLayer;

pub use view::shadow;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::configuration::{assets::AssetLoadingPlugin, setup::SetupPlugin};

pub mod prelude {
    pub use super::assets::*;
    pub use super::physics::*;
    pub use super::schedule::*;
    pub use super::view::*;
}

pub(super) fn plugin(app: &mut App) {
    // Setup and configuration
    app.add_plugins((physics::plugin, SetupPlugin, schedule::plugin, view::plugin));

    // Third-party plugins
    app.add_plugins((AssetLoadingPlugin, TilemapPlugin, BehavePlugin::default()));
}
