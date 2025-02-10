use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::FacingDirection,
    combat::{
        attributes::{mana::Mana, Health},
        components::{ActionState, AimPosition},
        damage::components::HasIFrames,
    },
    configuration::{
        assets::{SpriteAssets, SpriteSheetLayouts},
        GameCollisionLayer,
    },
    items::{
        equipment::{on_main_hand_activated, EquipEvent},
        inventory::Inventory,
        *,
    },
    labels::layer::ZLayer,
    movement::components::SimpleMotion,
    player::{systems::*, Player, PlayerStats},
};

pub fn spawn_player(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    texture_layouts: Res<SpriteSheetLayouts>,
) {
    //Player Inventory Setup
    let mut inventory = Inventory::default();
    inventory.add_item(spawn_health_potion(&mut commands)).ok();
    inventory
        .add_item(spawn_sword(&mut commands, &sprites))
        .ok();
    inventory.add_item(spawn_axe(&mut commands, &sprites)).ok();
    inventory
        .add_item(spawn_shovel(&mut commands, &sprites))
        .ok();
    inventory
        .add_item(spawn_ice_staff(&mut commands, &sprites, &texture_layouts))
        .ok();

    let player = commands
        .spawn((
            Player,
            PlayerStats::default(),
            AimPosition::default(),
            SimpleMotion::new(450.0),
            Health::new(100.0),
            Mana::new(100.0, 10.0),
            inventory,
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
            (FacingDirection::Down, ActionState::Idle),
            Transform::from_xyz(0., 0., ZLayer::Player.z()),
        ))
        .observe(death::on_player_defeated)
        .observe(on_main_hand_activated)
        .id();

    let starting_staff = spawn_fire_staff(&mut commands, &sprites, &texture_layouts);
    commands.trigger_targets(EquipEvent::new(starting_staff), player);

    info!("Player spawned: {}", player);
}
