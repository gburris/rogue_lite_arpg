use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::prelude::*;

/// Center of chest relative to its sprite's anchor point.
const CHEST_HEIGHT_OFFSET: f32 = -8.0;
const BOTTOM_OF_CHEST: f32 = CHEST_HEIGHT_OFFSET - 8.0;
/// Delay before coins pop: the open animation runs 9 frames at 0.1 s each,
/// so the burst lands as the lid finishes opening.
const COIN_DROP_DELAY_SECS: f32 = 0.9;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_spawn_chests);
}

#[derive(Event, Debug)]
pub struct SpawnChests(pub Vec<Vec2>);

#[derive(Component, Default, Clone)]
#[require(YSort::from_offset(BOTTOM_OF_CHEST), DespawnOnExit::<AppState>(AppState::Playing))]
struct Chest;

#[derive(Component, Default, Clone)]
#[require(
    Collider::rectangle(26.0, 8.0),
    RigidBody::Static,
    CollisionLayers::new(
        GameCollisionLayer::LowObstacle,
        GameCollisionLayer::LOW_OBSTACLE_FILTERS
    ),
    Transform::from_translation(Vec3::new(0.0, CHEST_HEIGHT_OFFSET, 0.0)),
    DespawnOnExit::<AppState>(AppState::Playing)
)]
struct ChestCollider;

fn on_spawn_chests(
    chest_spawn_trigger: On<SpawnChests>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    for &spawn_position in &chest_spawn_trigger.0 {
        commands.spawn_scene(chest(&sprites, &sprite_layouts, spawn_position));
    }
}

fn chest(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    spawn_position: Vec2,
) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        sprites.chests_sprite_sheet.clone(),
        TextureAtlas {
            layout: sprite_layouts.chest.clone(),
            index: 0,
        },
    );
    bsn! {
        Chest
        template(move |_| Ok(sprite.clone()))
        Anchor(Vec2::new(-0.18, 0.0))
        ~{AnimationIndices::OneShot(0..=8)}
        Transform {
            translation: {spawn_position.extend(ZLayer::OnGround.z())},
            scale: Vec3::new(2.0, 2.0, 1.0),
        }
        on(on_interaction_open_chest)
        Children [
            ChestCollider
            --
            ~InteractionZone::OPEN_CHEST
            Transform::from_xyz(0.0, CHEST_HEIGHT_OFFSET, 0.0)
        ]
    }
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

    commands.entity(chest_opened.zone).despawn();
    if let Ok(chest_transform) = chest_transforms.get(chest_entity) {
        commands
            .delayed()
            .secs(COIN_DROP_DELAY_SECS)
            .trigger(DropGold {
                amount: 999,
                location: chest_transform.translation.truncate(),
            });
    }
}
