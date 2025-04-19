use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    character::physical_collider,
    combat::{damage::hurtbox, Mana},
    configuration::{
        assets::{Shadows, SpriteAssets, SpriteSheetLayouts},
        shadow, GameCollisionLayer, CHARACTER_FEET_POS_OFFSET,
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
    shadows: Res<Shadows>,
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
            game_progress.base_stats.clone(),
            Sprite::from_atlas_image(
                sprites.player_sprite_sheet.clone(),
                TextureAtlas {
                    layout: atlases.player_atlas_layout.clone(),
                    ..default()
                },
            ),
            children![
                shadow(&shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
                physical_collider(),
                hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::AllyHurtBox),
                (
                    PlayerInteractionRadius,
                    Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0),
                    CollisionLayers::new(
                        [GameCollisionLayer::PlayerInteractionRadius],
                        [GameCollisionLayer::Interaction],
                    ),
                )
            ],
        ))
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
