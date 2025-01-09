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
        .add_plugins((
            // Core plugins group
            (
                SetupPlugin,
                SchedulePlugin,
                CollisionPlugin,
                DespawnPlugin,
                AssetLoadingPlugin,
            ),
            // Gameplay plugins group
            (TilemapPlugin, PlayerPlugin, EnemyPlugin, ProjectilePlugin),
            // Systems plugins group
            (
                StatusEffectPlugin,
                MapPlugin,
                MovementPlugin,
                ExperiencePlugin,
            ),
            // Additional plugins group
            (UIPlugin, NPCPlugin, DamagePlugin),
        ))
        .insert_resource(PlayerSize { x: 256.0, y: 256.0 })
        .run();
}
