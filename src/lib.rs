// Module declarations - keep these at the top
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

// External crate imports
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use console_error_panic_hook;
use wasm_bindgen::prelude::*;

// Local imports grouped by functionality
use crate::{
    // Core systems
    animation::AnimationPlugin,
    chests::plugin::ChestPlugin,

    combat::plugin::CombatPlugin,
    // Configuration and setup
    configuration::{assets::AssetLoadingPlugin, schedule::SchedulePlugin, setup::SetupPlugin},

    despawn::plugin::DespawnPlugin,
    enemy::plugin::EnemyPlugin,
    items::{equipment::plugin::EquipmentPlugin, grounded::plugin::GroundedPlugin},
    // Entity systems
    map::plugin::MapPlugin,
    movement::plugin::MovementPlugin,
    npc::NPCPlugin,
    player::plugin::PlayerPlugin,
    progression::plugin::ProgressionPlugin,

    // UI
    ui::plugin::UIPlugin,
};

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    App::new()
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
        ))
        // Entity systems
        .add_plugins((
            MapPlugin,
            EquipmentPlugin,
            GroundedPlugin,
            PlayerPlugin,
            EnemyPlugin,
            NPCPlugin,
            ChestPlugin,
        ))
        // UI
        .add_plugins(UIPlugin)
        .run();
}
