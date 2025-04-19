// In a new file, e.g., src/lib.rs or src/plugins.rs
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct GamePlugins;

use crate::{
    ai,
    animation::AnimationPlugin,
    combat::plugin::CombatPlugin,
    configuration::{assets::AssetLoadingPlugin, schedule::SchedulePlugin, setup::SetupPlugin},
    economy::EconomyPlugin,
    enemy::plugin::EnemyPlugin,
    items::{equipment::EquipmentPlugin, lootable::plugin::LootablePlugin},
    map::plugin::MapPlugin,
    npc::NPCPlugin,
    player::plugin::PlayerPlugin,
    progression::plugin::ProgressionPlugin,
    ui::plugin::UIPlugin,
    utility,
};

impl Plugin for GamePlugins {
    fn build(&self, app: &mut App) {
        app
            // Setup and configuration
            .add_plugins((SetupPlugin, AnimationPlugin, SchedulePlugin))
            // Third-party plugins
            .add_plugins((AssetLoadingPlugin, TilemapPlugin))
            // Core systems
            .add_plugins((
                utility::plugin,
                ai::plugin,
                CombatPlugin,
                ProgressionPlugin,
                EconomyPlugin,
                EquipmentPlugin,
            ))
            // Entity systems
            .add_plugins((
                MapPlugin,
                LootablePlugin,
                PlayerPlugin,
                EnemyPlugin,
                NPCPlugin,
            ))
            // UI
            .add_plugins(UIPlugin);
    }
}

// Optional: Create specialized plugin sets for different builds
#[cfg(target_arch = "wasm32")]
pub struct WasmPlugins;

#[cfg(target_arch = "wasm32")]
impl Plugin for WasmPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlugins);
        // Add any WASM-specific plugins here
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct NativePlugins;

#[cfg(not(target_arch = "wasm32"))]
impl Plugin for NativePlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlugins); // Add native-only plugins
    }
}
