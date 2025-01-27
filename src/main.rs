use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use baba_yaga::{
    animation::AnimationPlugin,
    chests::plugin::ChestPlugin,
    combat::plugin::CombatPlugin,
    configuration::{assets::AssetLoadingPlugin, schedule::SchedulePlugin, setup::SetupPlugin},
    despawn::plugin::DespawnPlugin,
    enemy::plugin::EnemyPlugin,
    items::equipment::plugin::EquipmentPlugin,
    map::plugin::MapPlugin,
    movement::plugin::MovementPlugin,
    npc::NPCPlugin,
    player::plugin::PlayerPlugin,
    progression::plugin::ProgressionPlugin,
    ui::{plugin::UIPlugin, PauseMenuPlugin},
};

fn main() {
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
            PlayerPlugin,
            EnemyPlugin,
            NPCPlugin,
            ChestPlugin,
        ))
        // UI plugins group
        .add_plugins((UIPlugin, PauseMenuPlugin))
        .run();
}
