use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use game_dev_project::{
    collision::CollisionPlugin,
    despawn::plugin::DespawnPlugin,
    enemy::plugin::EnemyPlugin,
    map::plugin::MapPlugin,
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
        .add_plugins((
            SetupPlugin,
            SchedulePlugin,
            CollisionPlugin,
            DespawnPlugin,
            AssetLoadingPlugin,
            TilemapPlugin,
            PlayerPlugin,
            EnemyPlugin,
            ProjectilePlugin,
            StatusEffectPlugin,
            MapPlugin,
            ExperiencePlugin,
            UIPlugin,
            NPCPlugin,
        ))
        .insert_resource(PlayerSize { x: 256.0, y: 256.0 })
        .run();
}
