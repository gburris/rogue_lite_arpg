use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{
        attributes::{mana::Mana, Health},
        damage::components::HasIFrames,
    },
    configuration::{assets::SpriteAssets, GameCollisionLayer},
    items::{
        equipment::{equipment_slots::EquipmentSlots, use_equipped},
        inventory::inventory::Inventory,
        spawn_fire_staff, spawn_health_potion, spawn_helmet, spawn_shovel, spawn_sword,
    },
    labels::layer::ZLayer,
    movement::components::SimpleMotion,
    player::{
        animation::components::PlayerAnimations, movement::MovementDirection, systems::*, Player,
        PlayerStats,
    },
};

#[derive(Component, Default)]
pub struct AimPosition {
    pub position: Vec2, // position where entitiy is aiming, for player this is the cursor
}

pub fn player_setup(mut commands: Commands, sprites: Res<SpriteAssets>) {
    //Player Inventory Setup
    let mut inventory = Inventory::default_inventory();
    let _ = inventory.add_item(spawn_health_potion(&mut commands));
    let _ = inventory.add_item(spawn_sword(&mut commands, &sprites));
    let _ = inventory.add_item(spawn_helmet(&mut commands, &sprites));
    let _ = inventory.add_item(spawn_shovel(&mut commands, &sprites));

    inventory
        .add_item(spawn_fire_staff(&mut commands, &sprites))
        .ok();

    commands
        .spawn((
            Player,
            PlayerStats::default(),
            AimPosition::default(),
            SimpleMotion::new(450.0),
            Health::new(100.0),
            Mana::new(100.0, 10.0),
            inventory,
            EquipmentSlots::default(),
            HasIFrames {
                duration: Duration::from_secs(1),
            },
            RigidBody::Dynamic,
            Collider::rectangle(100.0, 100.0),
            CollisionLayers::new(
                [GameCollisionLayer::Player, GameCollisionLayer::Grounded],
                [
                    GameCollisionLayer::Enemy,
                    GameCollisionLayer::Interaction,
                    GameCollisionLayer::InAir,
                    GameCollisionLayer::Grounded,
                    GameCollisionLayer::HighObstacle,
                    GameCollisionLayer::LowObstacle,
                ],
            ),
            LockedAxes::new().lock_rotation(),
            (MovementDirection::None, PlayerAnimations::IdleDown),
            Transform::from_xyz(0., 0., ZLayer::Player.z()),
        ))
        .observe(death::on_player_defeated)
        .observe(use_equipped::on_main_hand_activated);
}
