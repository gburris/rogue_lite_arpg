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
    character::CharacterPlugin,
    combat::CombatPlugin,
    configuration::{assets::AssetLoadingPlugin, setup::SetupPlugin},
    items,
    map::plugin::MapPlugin,
    progression::plugin::ProgressionPlugin,
    ui::plugin::UIPlugin,
    utility, world,
};

pub mod prelude {
    pub use super::schedule::*;
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Setup and configuration
            .add_plugins((SetupPlugin, animation::plugin, schedule::plugin))
            // Third-party plugins
            .add_plugins((AssetLoadingPlugin, TilemapPlugin))
            // Core systems
            .add_plugins((
                utility::plugin,
                CombatPlugin,
                ProgressionPlugin,
                world::plugin,
            ))
            // Entity systems
            .add_plugins((MapPlugin, items::plugin, CharacterPlugin))
            // UI
            .add_plugins(UIPlugin);
    }
}
