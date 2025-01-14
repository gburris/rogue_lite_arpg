use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::damage::components::Health,
    configuration::assets::SpriteAssets,
    helpers::labels::GameCollisionLayer,
    labels::states::{GameState, PlayingState},
    movement::components::SimpleMotion,
    player::{systems::death::on_player_defeated, Inventory, Item, Player, StatType},
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
