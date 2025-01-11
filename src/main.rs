use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use game_dev_project::{
    collision::CollisionPlugin,
    damage::DamagePlugin,
    despawn::plugin::DespawnPlugin,
    enemy::plugin::EnemyPlugin,
    map::plugin::MapPlugin,
    movement::plugin::MovementPlugin,
    npc::NPCPlugin,
    player::plugins::{ExperiencePlugin, PlayerPlugin},
    projectile::ProjectilePlugin,
    resources::{assets::AssetLoadingPlugin, PlayerSize},
    schedule::SchedulePlugin,
    setup::SetupPlugin,
    status_effects::plugin::StatusEffectPlugin,
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
        .add_plugins((UIPlugin, ExperiencePlugin))
        .insert_resource(PlayerSize { x: 256.0, y: 256.0 })
        .run();
}
