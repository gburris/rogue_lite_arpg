pub mod assets;
mod collision_layers;
#[cfg(feature = "dev")]
pub mod debug;
mod schedule;
pub mod setup;
mod view;

pub use collision_layers::GameCollisionLayer;
pub use view::CHARACTER_FEET_POS_OFFSET;
pub use view::YSort;
pub use view::ZLayer;

pub use view::shadow;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    animation,
    configuration::{assets::AssetLoadingPlugin, setup::SetupPlugin},
};

pub mod prelude {
    pub use super::schedule::*;
}

pub(super) fn plugin(app: &mut App) {
    // Setup and configuration
    app.add_plugins((SetupPlugin, animation::plugin, schedule::plugin));

    // Third-party plugins
    app.add_plugins((AssetLoadingPlugin, TilemapPlugin));
}
