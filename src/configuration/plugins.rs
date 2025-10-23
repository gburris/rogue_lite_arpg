use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct GamePlugin;

use crate::{
    animation,
    character::CharacterPlugin,
    combat::CombatPlugin,
    configuration::{assets::AssetLoadingPlugin, schedule::SchedulePlugin, setup::SetupPlugin},
    items,
    map::plugin::MapPlugin,
    progression::plugin::ProgressionPlugin,
    ui::plugin::UIPlugin,
    utility, world,
};

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Setup and configuration
            .add_plugins((SetupPlugin, animation::plugin, SchedulePlugin))
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
