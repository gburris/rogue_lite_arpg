use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    combat::damage::components::Health,
    enemy::{systems::on_enemy_defeated, Enemy},
    helpers::labels::GameCollisionLayer,
    map::resources::MapBounds,
    movement::components::{IsMoving, SimpleMotion},
    player::components::Player,
    resources::{assets::SpriteAssets, EnemySpawnConfig},
};

pub fn spawn_enemies_with_timer(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    time: Res<Time>,
    mut spawn_config: ResMut<EnemySpawnConfig>,
    mapbounds: Res<MapBounds>,
    player_transform_query: Query<&Transform, With<Player>>,
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
                commands
                    .spawn((
                        Enemy,
                        SimpleMotion::new(350.0),
                        IsMoving(true),
                        Health::new(30.),
                        RigidBody::Dynamic,
                        Collider::rectangle(100.0, 100.0),
                        // Currently enemies can only collide with projectiles
                        CollisionLayers::new(
                            GameCollisionLayer::Enemy,
                            [GameCollisionLayer::Projectile, GameCollisionLayer::Player],
                        ),
                        Sprite::from_image(sprites.merman_enemy.clone()),
                        Transform::from_xyz(spawn_position.x, spawn_position.y, 0.5),
                    ))
                    .observe(on_enemy_defeated);
            }
        }
    }
}
