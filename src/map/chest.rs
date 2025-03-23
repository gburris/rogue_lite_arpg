use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::animation::{AnimationIndices, AnimationTimer};
use crate::configuration::{GameCollisionLayer, YSort};

use crate::configuration::assets::{SpriteAssets, SpriteSheetLayouts};
use crate::econ::gold_drop::GoldDropEvent;
use crate::player::interact::{InteractionEvent, InteractionZone};

/// Center of chest relative to its sprite's anchor point
const CHEST_HEIGHT_OFFSET: f32 = -8.0;
const BOTTOM_OF_CHEST: f32 = CHEST_HEIGHT_OFFSET - 8.0;

#[derive(Debug, Event)]
pub struct SpawnChestsEvent(pub Vec<Vec2>);

#[derive(Component)]
#[require(YSort(|| YSort::from_offset(BOTTOM_OF_CHEST)))]
pub struct Chest;

#[derive(Component)]
#[require(
    Collider(|| Collider::rectangle(26.0, 8.0)),
    RigidBody(|| RigidBody::Static),
    CollisionLayers(|| CollisionLayers::new(GameCollisionLayer::LowObstacle, GameCollisionLayer::LOW_OBSTACLE_FILTERS))
)]
pub struct ChestCollider;

pub fn on_spawn_chests_event(
    chest_spawn_trigger: Trigger<SpawnChestsEvent>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    let chest_spawn_positions = chest_spawn_trigger.0.clone();
    for spawn_position in chest_spawn_positions {
        spawn_chest(&mut commands, &sprites, &sprite_layouts, spawn_position);
    }
}

fn spawn_chest(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    layouts: &SpriteSheetLayouts,
    spawn_position: Vec2,
) {
    commands
        .spawn((
            Chest,
            Sprite {
                image: sprites.chests_sprite_sheet.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: layouts.chest_layout.clone(),
                    index: 0,
                }),
                anchor: Anchor::Custom(Vec2::new(-0.18, 0.0)),
                ..default()
            },
            AnimationIndices {
                is_one_shot: true,
                first: 0,
                last: 8,
            },
            Transform {
                translation: spawn_position.extend(0.0),
                scale: Vec3::new(2.0, 2.0, 1.0),
                ..default()
            },
        ))
        .observe(on_interaction_open_chest)
        .with_children(|p| {
            p.spawn((
                ChestCollider,
                Transform::from_translation(Vec3::new(0.0, CHEST_HEIGHT_OFFSET, 0.0)),
            ));

            p.spawn((
                InteractionZone::OPEN_CHEST,
                Transform::from_translation(Vec3::new(0.0, CHEST_HEIGHT_OFFSET, 0.0)),
            ));
        });
}

pub fn on_interaction_open_chest(
    open_chest_trigger: Trigger<InteractionEvent>,
    chest_transforms: Query<&Transform, With<Chest>>,
    mut commands: Commands,
) {
    let chest_entity = open_chest_trigger.target();

    commands
        .entity(chest_entity)
        .insert(AnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )));

    commands
        .entity(open_chest_trigger.interaction_zone_entity)
        .despawn();

    if let Ok(chest_transform) = chest_transforms.get(chest_entity) {
        commands.trigger(GoldDropEvent {
            amount: 999,
            drop_location: chest_transform.translation.truncate(),
        });
    };
}
