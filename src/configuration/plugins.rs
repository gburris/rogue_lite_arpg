use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct GamePlugin;

use crate::{
    animation::AnimationPlugin,
    character::CharacterPlugin,
    combat::CombatPlugin,
    configuration::{assets::AssetLoadingPlugin, schedule::SchedulePlugin, setup::SetupPlugin},
    economy::EconomyPlugin,
    items,
    map::plugin::MapPlugin,
    progression::plugin::ProgressionPlugin,
    ui::plugin::UIPlugin,
    utility,
};

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Setup and configuration
            .add_plugins((SetupPlugin, AnimationPlugin, SchedulePlugin))
            // Third-party plugins
            .add_plugins((AssetLoadingPlugin, TilemapPlugin))
            // Core systems
            .add_plugins((
                utility::plugin,
                CombatPlugin,
                ProgressionPlugin,
                EconomyPlugin,
            ))
            // Entity systems
            .add_plugins((MapPlugin, items::plugin, CharacterPlugin))
            // UI
            .add_plugins(UIPlugin);
    }
}
