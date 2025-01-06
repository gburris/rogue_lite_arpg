use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use game_dev_project::{
    collision::CollisionPlugin,
    enemy::plugin::EnemyPlugin,
    map::plugin::MapPlugin,
    player::plugins::{ExperiencePlugin, PlayerPlugin},
    projectile::ProjectilePlugin,
    resources::{assets::AssetLoadingPlugin, PlayerSize},
    schedule::SchedulePlugin,
    setup::SetupPlugin,
    status_effects::StatusEffectPlugin,
    ui::plugin::UIPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            SetupPlugin,
            SchedulePlugin,
            CollisionPlugin,
            AssetLoadingPlugin,
            TilemapPlugin,
            PlayerPlugin,
            EnemyPlugin,
            ProjectilePlugin,
            StatusEffectPlugin,
            MapPlugin,
            ExperiencePlugin,
            UIPlugin,
        ))
        .insert_resource(PlayerSize { x: 256.0, y: 256.0 })
        .run();
}
