use avian2d::prelude::*;
use bevy::sprite::Anchor;
use bevy::{prelude::*, ui_widgets::observe};

use crate::prelude::*;

/// Center of chest relative to its sprite's anchor point
const CHEST_HEIGHT_OFFSET: f32 = -8.0;
const BOTTOM_OF_CHEST: f32 = CHEST_HEIGHT_OFFSET - 8.0;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_spawn_chests_event);

    app.add_observer(despawn_all::<CleanupZone, Chest>);
}

#[derive(Debug, Event)]
pub struct SpawnChestsEvent(pub Vec<Vec2>);

#[derive(Component)]
#[require(YSort::from_offset(BOTTOM_OF_CHEST))]
struct Chest;

#[derive(Component)]
#[require(
    Collider::rectangle(26.0, 8.0),
    RigidBody::Static,
    CollisionLayers::new(
        GameCollisionLayer::LowObstacle,
        GameCollisionLayer::LOW_OBSTACLE_FILTERS
    ),
    Transform::from_translation(Vec3::new(0.0, CHEST_HEIGHT_OFFSET, 0.0))
)]
struct ChestCollider;

fn on_spawn_chests_event(
    chest_spawn_trigger: On<SpawnChestsEvent>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    let chest_spawn_positions = chest_spawn_trigger.0.clone();
    for spawn_position in chest_spawn_positions {
        commands.spawn(chest(&sprites, &sprite_layouts, spawn_position));
    }
}

fn chest(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    spawn_position: Vec2,
) -> impl Bundle {
    (
        Chest,
        Sprite {
            image: sprites.chests_sprite_sheet.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.chest_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        Anchor(Vec2::new(-0.18, 0.0)),
        AnimationIndices::OneShot(0..=8),
        Transform {
            translation: spawn_position.extend(ZLayer::OnGround.z()),
            scale: Vec3::new(2.0, 2.0, 1.0),
            ..default()
        },
        children![
            ChestCollider,
            (
                InteractionZone::OPEN_CHEST,
                Transform::from_translation(Vec3::new(0.0, CHEST_HEIGHT_OFFSET, 0.0)),
            )
        ],
        observe(on_interaction_open_chest),
    )
}

fn on_interaction_open_chest(
    chest_opened: On<PlayerInteraction>,
    chest_transforms: Query<&Transform, With<Chest>>,
    mut commands: Commands,
) {
    let chest_entity = chest_opened.entity;

    commands
        .entity(chest_entity)
        .insert(AnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )));

    commands
        .entity(chest_opened.interaction_zone_entity)
        .despawn();

    if let Ok(chest_transform) = chest_transforms.get(chest_entity) {
        commands.trigger(GoldDrop {
            amount: 999,
            location: chest_transform.translation.truncate(),
        });
    };
}
