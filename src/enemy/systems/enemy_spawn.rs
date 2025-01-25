use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::attributes::Health,
    configuration::GameCollisionLayer,
    enemy::{systems::on_enemy_defeated, Enemy, EnemyAssets},
    map::systems::instance::spawn_instance_entities::EnemySpawnEvent,
    movement::components::{IsMoving, SimpleMotion},
};

pub fn spawn_enemies(
    enemy_trigger: Trigger<EnemySpawnEvent>,
    mut commands: Commands,
    enemy_assets: Res<EnemyAssets>,
    asset_server: Res<AssetServer>,
) {
    let enemy_spawn_positions = enemy_trigger.0.clone();
    for spawn_position in enemy_spawn_positions {
        spawn_enemy(
            &mut commands,
            "Merman",
            &enemy_assets,
            &asset_server,
            spawn_position,
        );
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    enemy_name: &str,
    enemy_assets: &Res<EnemyAssets>,
    asset_server: &Res<AssetServer>,
    spawn_position: Vec3,
) {
    if let Some(enemy) = enemy_assets.enemy_config.get(enemy_name) {
        commands
            .spawn((
                Enemy,
                SimpleMotion::new(enemy.simple_motion_speed),
                IsMoving(true),
                Health::new(enemy.health),
                LockedAxes::new().lock_rotation(),
                RigidBody::Dynamic,
                Collider::rectangle(enemy.collider_size.0, enemy.collider_size.1),
                CollisionLayers::new(
                    [GameCollisionLayer::Enemy],
                    [
                        GameCollisionLayer::Player,
                        GameCollisionLayer::InAir,
                        GameCollisionLayer::Grounded,
                        GameCollisionLayer::HighObstacle,
                        GameCollisionLayer::LowObstacle,
                    ],
                ),
                Sprite::from_image(asset_server.load(&enemy.sprite_path)),
                Transform::from_translation(spawn_position),
            ))
            .observe(on_enemy_defeated);
    } else {
        eprintln!("Enemy {} not found in enemy config.", enemy_name);
    }
}
