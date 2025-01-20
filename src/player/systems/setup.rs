use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::damage::components::{HasIFrames, Health},
    configuration::assets::SpriteAssets,
    helpers::labels::GameCollisionLayer,
    items::{spawn_health_potion, spawn_helmet, spawn_shovel, spawn_sword},
    labels::states::AppState,
    movement::components::SimpleMotion,
    player::{
        systems::death::on_player_defeated, Inventory, Player, PlayerEquipmentSlots, PlayerStats,
    },
};

pub fn player_setup(
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    sprites: Res<SpriteAssets>,
) {
    let mut inventory = Inventory::default_inventory();
    let _ = inventory.add_item(spawn_health_potion(&mut commands));
    let _ = inventory.add_item(spawn_sword(&mut commands, &sprites));
    let _ = inventory.add_item(spawn_helmet(&mut commands, &sprites));
    let _ = inventory.add_item(spawn_shovel(&mut commands, &sprites));

    commands
        .spawn((
            Player,
            PlayerStats::default(),
            SimpleMotion::new(600.0),
            LockedAxes::new().lock_rotation(),
            Health::new(100.0),
            inventory,
            HasIFrames {
                duration: Duration::from_secs(1),
            },
            PlayerEquipmentSlots::default(),
            RigidBody::Dynamic,
            Collider::rectangle(100.0, 100.0),
            CollisionLayers::new(
                GameCollisionLayer::Player,
                [
                    GameCollisionLayer::Npc,
                    GameCollisionLayer::Interaction,
                    GameCollisionLayer::Portal,
                    GameCollisionLayer::Enemy,
                    GameCollisionLayer::Wall,
                ],
            ),
            Sprite::from_image(sprites.skeleton_player.clone()),
            Transform::from_xyz(0., 0., 1.0),
        ))
        .observe(on_player_defeated);
    game_state.set(AppState::Playing);
}
