// In a new file, e.g., src/lib.rs or src/plugins.rs
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct GamePlugins;

use {
    crate::animation::AnimationPlugin,
    crate::chests::plugin::ChestPlugin,
    crate::combat::plugin::CombatPlugin,
    crate::configuration::{
        assets::AssetLoadingPlugin, schedule::SchedulePlugin, setup::SetupPlugin,
    },
    crate::despawn::plugin::DespawnPlugin,
    crate::econ::plugin::EconPlugin,
    crate::enemy::plugin::EnemyPlugin,
    crate::items::{equipment::EquipmentPlugin, grounded::plugin::GroundedPlugin},
    crate::map::plugin::MapPlugin,
    crate::movement::plugin::MovementPlugin,
    crate::npc::NPCPlugin,
    crate::player::plugin::PlayerPlugin,
    crate::progression::plugin::ProgressionPlugin,
    crate::ui::plugin::UIPlugin,
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
                DespawnPlugin,
                MovementPlugin,
                CombatPlugin,
                ProgressionPlugin,
                EconPlugin,
                EquipmentPlugin,
            ))
            // Entity systems
            .add_plugins((
                MapPlugin,
                GroundedPlugin,
                PlayerPlugin,
                EnemyPlugin,
                NPCPlugin,
                ChestPlugin,
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
