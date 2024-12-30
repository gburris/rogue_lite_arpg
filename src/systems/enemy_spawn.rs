use crate::components::Enemy;
use crate::components::Player;
use crate::components::Position;
use crate::resources::EnemySpawnConfig;
use crate::resources::MapBounds;
use bevy::prelude::*;
use rand::Rng;

pub fn spawn_enemies_with_timer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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

            let mut spawn_position = Position { x, y };
            if let Ok(player_position) = player_transform_query.get_single() {
                let player_transform = &player_position;
                let player_pos = player_transform.translation;
                let distance = ((spawn_position.x - player_pos.x).powi(2)
                    + (spawn_position.y - player_pos.y).powi(2))
                .sqrt();
                if distance > 15.0 {
                    spawn_position.x -= 30.0;
                    spawn_position.y -= 30.0;
                    commands.spawn((
                        Enemy {
                            health: 10.0,
                            speed: 1.0,
                            position: spawn_position,
                        },
                        Sprite::from_image(asset_server.load("merman.png")),
                        Transform::from_xyz(x, y, 0.5),
                    ));
                    return;
                } else {
                    commands.spawn((
                        Enemy {
                            health: 10.0,
                            speed: 1.0,
                            position: spawn_position,
                        },
                        Sprite::from_image(asset_server.load("merman.png")),
                        Transform::from_xyz(x, y, 0.5),
                    ));
                    return;
                }
            } else {
                commands.spawn((
                    Enemy {
                        health: 10.0,
                        speed: 1.0,
                        position: spawn_position,
                    },
                    Sprite::from_image(asset_server.load("merman.png")),
                    Transform::from_xyz(x, y, 0.5),
                ));
                return;
            }
        }
    }
}
