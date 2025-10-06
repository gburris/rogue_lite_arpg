use avian2d::prelude::{RayCaster, SpatialQueryFilter};
use bevy::{prelude::*, ui_widgets::observe};
use bevy_behave::prelude::*;

mod defeat;

use crate::{
    character::{
        Character,
        behavior::{Anchor, AttemptMelee, Chase, Idle, KeepDistanceAndFire, Retreat, Wander},
        physical_collider,
        vision::{VisionCapabilities, Watching},
    },
    combat::{Health, Mana, damage::hurtbox},
    configuration::{
        CHARACTER_FEET_POS_OFFSET, GameCollisionLayer,
        assets::{Shadows, SpriteAssets, SpriteSheetLayouts},
        shadow,
    },
    economy::Purse,
    items::{
        Items,
        equipment::{Equipped, on_equipment_activated},
        fire_staff, health_potion, ice_staff, sword,
    },
    map::EnemiesSpawnEvent,
    prelude::*,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_enemies);
    }
}

#[derive(Component)]
#[require(Character, Experience, VisionCapabilities, Purse { amount: 50 })]
pub struct Enemy;

//Experience granted by the enemy when player defeats it
#[derive(Component)]
pub struct Experience {
    pub base_exp: f32,
}

impl Default for Experience {
    fn default() -> Self {
        Experience { base_exp: 10.0 }
    }
}

#[derive(Debug, Clone)]
pub struct EnemySpawnData {
    pub position: Vec2,
    pub enemy_type: EnemyType,
}

#[derive(Component, PartialEq, Clone, Debug)]
pub enum EnemyType {
    Warrior,
    IceMage,
    FireMage,
}

fn spawn_enemies(
    enemy_trigger: On<EnemiesSpawnEvent>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
    player: Single<Entity, With<Player>>,
) {
    for spawn_data in enemy_trigger.0.clone() {
        spawn_enemy(
            &mut commands,
            spawn_data,
            &sprites,
            &sprite_layouts,
            &shadows,
            player.entity(),
        );
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    spawn_data: EnemySpawnData,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
    player: Entity,
) {
    let chase_behavior = behave! {
        Behave::While => {
            Behave::spawn_named("Chase", Chase),
            Behave::trigger(AttemptMelee)
        }
    };

    let melee_enemy_behavior = behave! {
        Behave::Forever => {
            Behave::Fallback => {
                Behave::Sequence => {
                    Behave::spawn_named("Wander", Wander::builder().timer_range(1.0..2.0)),
                    Behave::spawn_named("Idle", Idle::default().timer_range(3.0..5.0)),
                },
                Behave::spawn_named("Retreat", Retreat),
                @chase_behavior
            }
        }
    };

    let ranged_enemy_behavior = behave! {
        Behave::Forever => {
            Behave::Fallback => {
                Behave::Sequence => {
                    Behave::spawn_named("Wander", Wander::builder().timer_range(1.0..2.0)),
                    Behave::spawn_named("Idle", Idle::default().timer_range(3.0..5.0)),
                },
                Behave::spawn_named("Retreat", Retreat),
                Behave::spawn_named("Keep distance and fire", KeepDistanceAndFire)
            }
        }
    };

    match spawn_data.enemy_type {
        EnemyType::Warrior => commands.spawn((
            warrior(sprites, sprite_layouts),
            base_enemy(spawn_data.position, player),
            enemy_children(melee_enemy_behavior, &shadows),
        )),

        EnemyType::IceMage => commands.spawn((
            ice_mage(sprites, sprite_layouts),
            base_enemy(spawn_data.position, player),
            enemy_children(ranged_enemy_behavior, &shadows),
        )),

        EnemyType::FireMage => commands.spawn((
            fire_mage(sprites, sprite_layouts),
            base_enemy(spawn_data.position, player),
            enemy_children(ranged_enemy_behavior, &shadows),
        )),
    };
}

fn base_enemy(position: Vec2, player: Entity) -> impl Bundle {
    (
        Enemy,
        Transform::from_translation(position.extend(0.0)),
        Anchor::new(position, 256.0), // 8 tile radius
        Mana::new(100.0, 10.0),
        // enemy vision distance
        RayCaster::default()
            .with_max_distance(350.0)
            .with_query_filter(SpatialQueryFilter::from_mask([
                GameCollisionLayer::AllyHurtBox,
                GameCollisionLayer::HighObstacle,
            ]))
            .with_max_hits(1),
        Watching(player),
        observe(defeat::on_enemy_defeated),
        observe(on_equipment_activated),
    )
}

fn enemy_children(behavior: Tree<Behave>, shadows: &Shadows) -> impl Bundle {
    children![
        shadow(&shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
        physical_collider(),
        hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::EnemyHurtBox),
        BehaveTree::new(behavior.clone()),
    ]
}

fn warrior(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        SimpleMotion::new(200.0),
        Health::new(40.0),
        Sprite::from_atlas_image(
            sprites.warrior_enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.enemy_atlas_layout.clone(),
                ..default()
            },
        ),
        related!(Items[(Equipped, sword(sprites)), health_potion(sprites)]),
    )
}

fn ice_mage(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        SimpleMotion::new(100.0),
        Health::new(20.0),
        Sprite::from_atlas_image(
            sprites.ice_mage_enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.enemy_atlas_layout.clone(),
                ..default()
            },
        ),
        related!(Items[(Equipped, ice_staff(sprites, sprite_layouts)), health_potion(sprites)]),
    )
}

fn fire_mage(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        SimpleMotion::new(150.0),
        Health::new(20.0),
        Sprite::from_atlas_image(
            sprites.fire_mage_enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.enemy_atlas_layout.clone(),
                ..default()
            },
        ),
        related!(Items[(Equipped, fire_staff(sprites, sprite_layouts)), health_potion(sprites)]),
    )
}
