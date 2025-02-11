use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::FacingDirection,
    combat::{attributes::mana::Mana, components::ActionState, damage::components::HasIFrames},
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
    player::{systems::*, Player},
};

pub fn spawn_player(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    texture_layouts: Res<SpriteSheetLayouts>,
) {
    //Player Inventory Setup
    let main_hand = spawn_fire_staff(&mut commands, &sprites, &texture_layouts);

    let starting_items = [
        spawn_health_potion(&mut commands, &sprites),
        spawn_sword(&mut commands, &sprites),
        spawn_shovel(&mut commands, &sprites),
        spawn_ice_staff(&mut commands, &sprites, &texture_layouts),
        main_hand,
    ];

    let player = commands
        .spawn((
            Player,
            Inventory::new(&starting_items.into(), 0),
            Mana::new(100.0, 10.0),
            HasIFrames {
                duration: Duration::from_secs(1),
            },
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
                    GameCollisionLayer::Magnet,
                ],
            ),
            (FacingDirection::Down, ActionState::Idle),
            Transform::from_xyz(0., 0., ZLayer::Player.z()),
        ))
        .add_children(&starting_items)
        .observe(death::on_player_defeated)
        .observe(on_main_hand_activated)
        .id();

    commands.trigger_targets(EquipEvent::new(main_hand), player);

    info!("Player spawned: {}", player);
}
