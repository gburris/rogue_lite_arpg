use crate::components::Enemy;
use crate::components::Position;
use bevy::prelude::*;

pub fn spawn_enemies(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Spawning Enemy");
    commands.spawn((
        Enemy {
            health: 10.0,
            speed: 1.0,
            position: Position { x: 100.0, y: 100.0 },
        },
        Sprite::from_image(asset_server.load("merman.png")),
        Transform::from_xyz(0., 0., 0.5),
    ));
}
