use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::{
    components::{Health, HealthBar, Speed},
    labels::states::GameState,
    player::Player,
    resources::assets::SpriteAssets,
};

pub fn player_setup(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    sprites: Res<SpriteAssets>,
) {
    commands.spawn((
        Player,
        Speed::default(),
        Health::default(),
        HealthBar {
            health_percetange: 100.0,
        },
        RigidBody::Dynamic,
        Collider::rectangle(100.0, 100.0),
        Sprite::from_image(sprites.skeleton_player.clone()),
        Transform::from_xyz(0., 0., 1.0),
    ));

    game_state.set(GameState::Playing);
}
