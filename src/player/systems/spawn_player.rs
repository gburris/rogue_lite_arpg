use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{
        attributes::{mana::Mana, Health},
        damage::components::HasIFrames,
    },
    configuration::{
        assets::{SpriteAssets, SpriteSheetLayouts},
        GameCollisionLayer,
    },
    items::{
        equipment::{equipment_slots::EquipmentSlots, use_equipped},
        inventory::inventory::Inventory,
        spawn_axe, spawn_fire_staff, spawn_health_potion, spawn_helmet, spawn_ice_staff,
        spawn_shovel, spawn_sword,
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

//This is used to decide if we should update the item transform to keep up with the player moving
//Or leave it alone during an attack animation
#[derive(Component, Debug, Hash, Eq, PartialEq, Clone)]
pub enum CurrentActionState {
    Attacking, //Sword is swinging
    None,      //Moving
}

pub fn spawn_player(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    texture_layouts: Res<SpriteSheetLayouts>,
) {
    //Player Inventory Setup
    let mut inventory = Inventory::default_inventory();
    inventory.add_item(spawn_health_potion(&mut commands)).ok();
    inventory
        .add_item(spawn_sword(&mut commands, &sprites))
        .ok();
    inventory.add_item(spawn_axe(&mut commands, &sprites)).ok();
    inventory
        .add_item(spawn_helmet(&mut commands, &sprites))
        .ok();
    inventory
        .add_item(spawn_shovel(&mut commands, &sprites))
        .ok();
    inventory
        .add_item(spawn_ice_staff(&mut commands, &sprites, &texture_layouts))
        .ok();

    let fire_staff = spawn_fire_staff(&mut commands, &sprites, &texture_layouts);

    // TODO: Fix it such that any equipped weapon is made visible
    commands.entity(fire_staff).insert(Visibility::Visible);

    commands
        .spawn((
            Player,
            PlayerStats::default(),
            AimPosition::default(),
            SimpleMotion::new(450.0),
            Health::new(100.0),
            Mana::new(100.0, 10.0),
            inventory,
            EquipmentSlots {
                mainhand: Some(fire_staff), // start with fire staff equipped
                head: None,
            },
            HasIFrames {
                duration: Duration::from_secs(1),
            },
            RigidBody::Dynamic,
            Collider::rectangle(40.0, 50.0),
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
            (
                MovementDirection::None,
                CurrentActionState::None,
                PlayerAnimations::IdleDown,
            ),
            Transform::from_xyz(0., 0., ZLayer::Player.z()),
        ))
        .observe(death::on_player_defeated)
        .observe(use_equipped::on_main_hand_activated)
        .add_child(fire_staff);
}
