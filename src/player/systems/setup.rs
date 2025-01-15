use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::damage::components::Health,
    helpers::labels::GameCollisionLayer,
    items::get_default_staff,
    labels::states::{GameState, PlayingState},
    movement::components::SimpleMotion,
    player::{systems::death::on_player_defeated, Inventory, Player},
    resources::assets::SpriteAssets,
};

pub fn player_setup(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut playing_state: ResMut<NextState<PlayingState>>,
    sprites: Res<SpriteAssets>,
) {
    let mut inventory = Inventory::default_inventory();
    inventory.add_item(get_default_staff());

    commands
        .spawn((
            Player,
            SimpleMotion::new(600.0),
            LockedAxes::new().lock_rotation(),
            Health::new(100.0),
            inventory,
            RigidBody::Dynamic,
            Collider::rectangle(100.0, 100.0),
            CollisionLayers::new(
                GameCollisionLayer::Player,
                [
                    GameCollisionLayer::Npc,
                    GameCollisionLayer::Interaction,
                    GameCollisionLayer::Portal,
                    GameCollisionLayer::Enemy,
                ],
            ),
            Sprite::from_image(sprites.skeleton_player.clone()),
            Transform::from_xyz(0., 0., 1.0),
        ))
        .observe(on_player_defeated);
    playing_state.set(PlayingState::BeforeRun);
    game_state.set(GameState::Playing);
}
