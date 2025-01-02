use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::components::WarpZone;

pub fn warpzone_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        WarpZone::default(), //Default is LevelTwo, Since we begin on LevelOne
        RigidBody::Static,
        Collider::rectangle(100.0, 100.0),
        Sprite::from_image(asset_server.load("warpzone.png")),
        Transform::from_xyz(500.0, 500.0, 1.0),
    ));
}
