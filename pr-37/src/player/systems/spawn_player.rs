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
        equipment::{on_main_hand_activated, Equipped},
        inventory::Inventory,
        *,
    },
    labels::layer::ZLayer,
    player::{systems::*, Player, PlayerStats},
    progression::GameProgress,
};

pub fn spawn_player(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    texture_layouts: Res<SpriteSheetLayouts>,
    game_progress: Res<GameProgress>,
) {
    let starting_items = [
        spawn_fire_staff(&mut commands, &sprites, &texture_layouts),
        spawn_health_potion(&mut commands, &sprites),
        spawn_sword(&mut commands, &sprites),
    ];
    let current_player_base_stats = PlayerStats::from(game_progress.base_stats);
    let player = commands
        .spawn((
            Player,
            Inventory::builder()
                .items(starting_items.into())
                .coins(0)
                .max_capacity(10)
                .build(),
            Mana::new(100.0, 10.0),
            HasIFrames {
                duration: Duration::from_secs(1),
            },
            current_player_base_stats,
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

    commands
        .entity(starting_items[0])
        .insert(Equipped::new(player));

    info!("Player spawned: {}", player);
}
