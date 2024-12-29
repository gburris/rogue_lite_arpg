use bevy::prelude::*;

use crate::components::{Player, Position};

pub fn player_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d); //This should not be here
    commands.spawn((
        Player {
            speed: 10.0,
            position: Position { x: 100.0, y: 100.0 },
        },
        Sprite::from_image(asset_server.load("skeleton.png")),
        Transform::from_xyz(0., 0., 0.),
    ));
}
