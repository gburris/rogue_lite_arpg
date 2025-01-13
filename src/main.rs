use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use game_dev_project::{
    collision::CollisionPlugin,
    combat::damage::DamagePlugin,
    combat::projectile::ProjectilePlugin,
    combat::status_effects::plugin::StatusEffectPlugin,
    despawn::plugin::DespawnPlugin,
    enemy::plugin::EnemyPlugin,
    map::plugin::MapPlugin,
    movement::plugin::MovementPlugin,
    npc::NPCPlugin,
    player::plugin::PlayerPlugin,
    resources::{assets::AssetLoadingPlugin, PlayerSize},
    schedule::SchedulePlugin,
    setup::SetupPlugin,
    ui::plugin::UIPlugin,
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
            DamagePlugin,
            StatusEffectPlugin,
        ))
        // Entity-domain plugins (map, player, enemy, npc, etc..)
        .add_plugins((
            MapPlugin,
            PlayerPlugin,
            EnemyPlugin,
            ProjectilePlugin,
            NPCPlugin,
        ))
        // Additional plugins group
        .add_plugins(UIPlugin)
        .insert_resource(PlayerSize { x: 256.0, y: 256.0 })
        .run();
}
