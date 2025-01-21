use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use game_dev_project::{
    collision::CollisionPlugin,
    combat::plugin::CombatPlugin,
    configuration::{assets::AssetLoadingPlugin, schedule::SchedulePlugin, setup::SetupPlugin},
    despawn::plugin::DespawnPlugin,
    enemy::plugin::EnemyPlugin,
    map::plugin::MapPlugin,
    movement::plugin::MovementPlugin,
    npc::NPCPlugin,
    player::plugin::PlayerPlugin,
    progression::plugin::ProgressionPlugin,
    ui::{plugin::UIPlugin, PauseMenuPlugin},
};

fn main() {
    App::new()
        .add_plugins((SetupPlugin, SchedulePlugin))
        .add_plugins((AssetLoadingPlugin, TilemapPlugin)) // 3rd party crates
        // Core plugins
        .add_plugins((
            CollisionPlugin,
            DespawnPlugin,
            MovementPlugin,
            CombatPlugin,
            ProgressionPlugin,
        ))
        // Entity-domain plugins (map, player, enemy, npc, etc..)
        .add_plugins((MapPlugin, PlayerPlugin, EnemyPlugin, NPCPlugin))
        // UI plugins group
        .add_plugins((UIPlugin, PauseMenuPlugin))
        .run();
}
