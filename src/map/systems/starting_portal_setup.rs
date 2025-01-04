use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::{
    labels::states::GameState, map::components::StartingPortal, resources::assets::SpriteAssets,
};

pub fn starting_portal_setup(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    sprites: Res<SpriteAssets>,
) {
    commands.spawn((
        StartingPortal,
        RigidBody::Static,
        Collider::rectangle(100.0, 100.0),
        Sprite::from_image(sprites.run_start_portal.clone()),
        Transform::from_xyz(500.0, 500.0, 1.0),
    ));
    game_state.set(GameState::SpawnPlayer);
}
