use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::components::{Health, HealthBar, Player, Speed};

pub fn player_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        Speed::default(),
        Health::default(),
        HealthBar {
            health_percetange: 100.0,
        },
        RigidBody::Dynamic,
        Collider::rectangle(100.0, 100.0),
        Sprite::from_image(asset_server.load("skeleton.png")),
        Transform::from_xyz(0., 0., 1.0),
    ));
}
