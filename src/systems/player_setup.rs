use bevy::prelude::*;

use crate::components::{Collider, Health, HealthBar, Player, Speed};

pub fn player_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    // commands.spawn(Camera2d); //This should not be here
    commands.spawn((
        Player,
        Speed::default(),
        Health::default(),
        HealthBar {
            health_percetange: 100.0,
        },
        Collider {
            size: Vec2::new(100.0, 100.0),
        },
        Sprite::from_image(asset_server.load("skeleton.png")),
        Transform::from_xyz(0., 0., 0.),
    ));
}
