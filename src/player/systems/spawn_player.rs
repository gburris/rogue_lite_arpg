use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{invulnerable::HasIFrames, Mana},
    configuration::{
        assets::{SpriteAssets, SpriteSheetLayouts},
        GameCollisionLayer, ZLayer,
    },
    items::{
        equipment::{on_equipment_activated, on_equipment_deactivated, Equipped},
        inventory::Inventory,
        *,
    },
    player::{interact::PlayerInteractionRadius, systems::*, Player},
    progression::GameProgress,
};

pub fn spawn_player(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    texture_layouts: Res<SpriteSheetLayouts>,
    game_progress: Res<GameProgress>,
    atlases: Res<SpriteSheetLayouts>,
) {
    let starting_items = [
        spawn_fire_staff(&mut commands, &sprites, &texture_layouts),
        spawn_health_potion(&mut commands, &sprites),
        spawn_sword(&mut commands, &sprites),
        spawn_offhand(&mut commands, &sprites, &texture_layouts, "tome_of_healing"),
        spawn_offhand(&mut commands, &sprites, &texture_layouts, "magic_shield"),
        spawn_offhand(&mut commands, &sprites, &texture_layouts, "knight_shield"),
    ];

    let player = commands
        .spawn((
            Player::default(),
            Inventory::builder()
                .items(starting_items.into())
                .coins(0)
                .max_capacity(50)
                .build(),
            Mana::new(100.0, 10.0),
            HasIFrames {
                duration: Duration::from_secs(1),
            },
            game_progress.base_stats.clone(),
            // Collider::rectangle(40.0, 50.0),
            // Sensor,
            // CollisionLayers::new(
            //     [GameCollisionLayer::Player],
            //     [
            //         GameCollisionLayer::Enemy,
            //         GameCollisionLayer::Interaction,
            //         GameCollisionLayer::InAir,
            //         GameCollisionLayer::Grounded,
            //         GameCollisionLayer::HighObstacle,
            //         GameCollisionLayer::LowObstacle,
            //         GameCollisionLayer::Magnet,
            //     ],
            // ),
            Transform::from_xyz(0., 0., ZLayer::OnGround.z()),
            Sprite::from_atlas_image(
                sprites.player_sprite_sheet.clone(),
                TextureAtlas {
                    layout: atlases.player_atlas_layout.clone(),
                    ..default()
                },
            ),
        ))
        .with_children(|spawner| {
            // collider to bump into stuff
            spawner.spawn((
                Transform::from_xyz(0.0, -20.0, 0.0),
                Collider::circle(12.0),
                CollisionLayers::new(
                    [GameCollisionLayer::Grounded],
                    [
                        GameCollisionLayer::Grounded,
                        GameCollisionLayer::HighObstacle,
                        GameCollisionLayer::LowObstacle,
                    ],
                ),
            ));

            // hitbox

            // player interaction radius
            spawner.spawn((
                PlayerInteractionRadius,
                Transform::from_xyz(0.0, -20.0, 0.0),
                Collider::circle(20.0),
                Sensor,
                CollidingEntities::default(),
                CollisionLayers::new(
                    [GameCollisionLayer::Player],
                    [GameCollisionLayer::Interaction, GameCollisionLayer::Magnet],
                ),
            ));
        })
        .add_children(&starting_items)
        .observe(death::on_player_defeated)
        .observe(on_equipment_activated)
        .observe(on_equipment_deactivated)
        .id();

    commands
        .entity(starting_items[0])
        .insert(Equipped::new(player));

    info!("Player spawned: {}", player);
}
