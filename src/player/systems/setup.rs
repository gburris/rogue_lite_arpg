use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::prelude::*;

use crate::{
    components::{Health, HealthBar, Speed},
    helpers::labels::GameCollisionLayer,
    labels::states::{GameState, PlayingState},
    player::Player,
    resources::assets::SpriteAssets,
};

pub fn player_setup(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut playing_state: ResMut<NextState<PlayingState>>,
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
        CollisionLayers::new(
            GameCollisionLayer::Player,
            [
                GameCollisionLayer::Npc,
                GameCollisionLayer::Interaction,
                GameCollisionLayer::Portal,
            ],
        ),
        Sprite::from_image(sprites.skeleton_player.clone()),
        Transform::from_xyz(0., 0., 1.0),
    ));
    playing_state.set(PlayingState::BeforeRun);
    game_state.set(GameState::Playing);
}
