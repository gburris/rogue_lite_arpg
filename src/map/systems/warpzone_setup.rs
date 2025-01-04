use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::{labels::states::GameState, map::WarpZone, resources::assets::SpriteAssets};

pub fn warpzone_setup(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    commands.spawn((
        WarpZone::default(), //Default is LevelTwo, Since we begin on LevelOne
        RigidBody::Static,
        Collider::rectangle(100.0, 100.0),
        Sprite::from_image(sprites.warp_zone.clone()),
        Transform::from_xyz(500.0, 500.0, 1.0),
    ));
    game_state.set(GameState::PlayingInZone);
}
