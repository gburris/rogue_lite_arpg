use avian2d::prelude::{
    AngularDamping, AngularVelocity, Collider, CollisionLayers, LinearDamping, LinearVelocity,
    RigidBody,
};
use bevy::prelude::*;

use crate::{
    components::{Health, HealthBar, Speed},
    helpers::labels::GameCollisionLayer,
    labels::states::{GameState, PlayingState},
    player::{Inventory, Item, Player, StatType},
    resources::assets::SpriteAssets,
};

pub fn player_setup(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut playing_state: ResMut<NextState<PlayingState>>,
    sprites: Res<SpriteAssets>,
) {
    let mut staff = Item::new("Staff of Casting");
    staff.add_stat(StatType::SpellPower, 10);
    staff.add_stat(StatType::CastSpeed, 10);

    // Create a new player inventory and add the staff to it
    let mut inventory = Inventory::default_inventory();

    // Add the staff to the inventory
    match inventory.add_item(staff) {
        Ok(_) => println!("Staff added to inventory!"),
        Err(err) => println!("Error: {}", err),
    };

    commands.spawn((
        Player,
        Speed {
            velocity: 10000.,
            ..Default::default()
        },
        Health::default(),
        HealthBar {
            health_percetange: 100.0,
        },
        LinearDamping(1000.), //Stop the player from sliding on the ground
        AngularDamping(1000.),
        AngularVelocity(0.0), //Stop the player from ever "Spinning"
        LinearVelocity::default(),
        inventory,
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
