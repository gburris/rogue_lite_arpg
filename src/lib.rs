pub mod animation;
pub mod chests;
pub mod combat;
pub mod configuration;
pub mod despawn;
pub mod econ;
pub mod enemy;
pub mod items;
pub mod labels;
pub mod map;
pub mod movement;
pub mod npc;
pub mod player;
pub mod progression;
pub mod ui;

use bevy::prelude::*;
use wasm_bindgen::prelude::*;

// This is the entry point for WASM
#[wasm_bindgen(start)]
pub fn start() {
    // Enable better error messages for WASM
    console_error_panic_hook::set_once();

    // Start your Bevy app - this should be similar to your main.rs setup
    App::new()
        .add_plugins((SetupPlugin, AnimationPlugin, SchedulePlugin))
        .add_plugins((AssetLoadingPlugin, TilemapPlugin)) // 3rd party crates
        // Core plugins
        .add_plugins((
            DespawnPlugin,
            MovementPlugin,
            CombatPlugin,
            ProgressionPlugin,
        ))
        // Entity-domain plugins (map, player, enemy, npc, etc..)
        .add_plugins((
            MapPlugin,
            EquipmentPlugin,
            GroundedPlugin,
            PlayerPlugin,
            EnemyPlugin,
            NPCPlugin,
            ChestPlugin,
        ))
        // UI plugins group
        .add_plugins(UIPlugin)
        .run();
}
