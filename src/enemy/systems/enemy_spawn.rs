use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    combat::attributes::Health,
    enemy::{resources::EnemySpawnConfig, systems::on_enemy_defeated, Enemy, EnemyAssets},
    helpers::labels::GameCollisionLayer,
    labels::layer::ZLayer,
    map::resources::MapBounds,
    movement::components::{IsMoving, SimpleMotion},
    player::components::Player,
};

pub fn spawn_enemies_with_timer(
    mut commands: Commands,
    time: Res<Time>,
    enemy_assets: Res<EnemyAssets>,
    mut spawn_config: ResMut<EnemySpawnConfig>,
    mapbounds: Res<MapBounds>,
    player_transform_query: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
) {
    spawn_config.timer.tick(time.delta());
    if spawn_config.timer.just_finished() {
        let mut rng = rand::thread_rng();
        for _ in 0..spawn_config.quantity {
            let x = rng.gen_range(mapbounds.min_x..mapbounds.max_x);
            let y = rng.gen_range(mapbounds.min_y..mapbounds.max_y);

            let mut spawn_position = Vec2::new(x, y);
            if let Ok(player_position) = player_transform_query.get_single() {
                let player_transform = &player_position;
                let player_pos = player_transform.translation;
                let distance = ((spawn_position.x - player_pos.x).powi(2)
                    + (spawn_position.y - player_pos.y).powi(2))
                .sqrt();
                if distance <= 15.0 {
                    spawn_position.x -= 30.0;
                    spawn_position.y -= 30.0;
                }
                spawn_enemy(
                    &mut commands,
                    "Merman",
                    &enemy_assets,
                    &asset_server,
                    spawn_position,
                );
            }
        }
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    enemy_name: &str,
    enemy_assets: &Res<EnemyAssets>,
    asset_server: &Res<AssetServer>,
    spawn_position: Vec2,
) {
    if let Some(enemy) = enemy_assets.enemy_config.get(enemy_name) {
        commands
            .spawn((
                Enemy,
                SimpleMotion::new(enemy.simple_motion_speed),
                IsMoving(true),
                Health::new(enemy.health),
                RigidBody::Dynamic,
                Collider::rectangle(enemy.collider_size.0, enemy.collider_size.1),
                CollisionLayers::new(
                    GameCollisionLayer::Enemy,
                    [
                        GameCollisionLayer::Projectile,
                        GameCollisionLayer::Player,
                        GameCollisionLayer::Wall,
                        GameCollisionLayer::Water,
                    ],
                ),
                Sprite::from_image(asset_server.load(&enemy.sprite_path)),
                Transform::from_xyz(spawn_position.x, spawn_position.y, ZLayer::Enemy.z()),
            ))
            .observe(on_enemy_defeated);
    } else {
        eprintln!("Enemy {} not found in enemy config.", enemy_name);
    }
}
