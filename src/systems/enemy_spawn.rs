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
    mapbounds: Res<MapBounds>
) {
    println!("Spawning enemy");
        spawn_config.timer.tick(time.delta());
        if spawn_config.timer.just_finished() {
            println!("Spawning enemy");
            println!("Timer just finished, spawning enemies");
            let mut rng = rand::thread_rng();

            for _ in 0..spawn_config.quantity {
                let x = rng.gen_range(mapbounds.min_x..mapbounds.max_x);
                let y = rng.gen_range(mapbounds.min_y..mapbounds.max_y);

                let spawn_position = Position { x, y };
                commands.spawn((
                    Enemy {
                        health: 10.0,
                        speed: 1.0,
                        position: spawn_position,
                    },
                    Sprite::from_image(asset_server.load("merman.png")),
                    Transform::from_xyz(x, y, 0.5),
                ));
            }
        
    }
}
