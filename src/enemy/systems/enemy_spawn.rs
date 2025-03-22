use avian2d::prelude::*;
use bevy::prelude::*;
use serde::Serialize;

use crate::{
    ai::{
        state::{ActionState, AimPosition, FacingDirection},
        SimpleMotion,
    },
    animation::{AnimationTimer, DefaultAnimationConfig},
    combat::{Health, Mana},
    configuration::{
        assets::{SpriteAssets, SpriteSheetLayouts},
        GameCollisionLayer,
    },
    enemy::{systems::on_enemy_defeated, Enemy, EnemyAssets},
    items::{
        equipment::{on_equipment_activated, Equipped},
        inventory::Inventory,
        spawn_health_potion, spawn_mainhand_weapon,
    },
    map::EnemiesSpawnEvent,
};

#[derive(Debug, Clone)]
pub struct EnemySpawnData {
    pub position: Vec3,
    pub enemy_type: EnemyType,
}

#[derive(Debug, Clone, Serialize, Component, Copy)]
pub enum EnemyType {
    IceMage,
    Warrior,
    FireMage,
}

impl EnemyType {
    pub fn name(&self) -> String {
        match self {
            Self::IceMage => "IceMage".to_owned(),
            Self::Warrior => "Warrior".to_owned(),
            Self::FireMage => "FireMage".to_owned(),
        }
    }

    pub fn sprite(&self, sprites: &SpriteAssets) -> Handle<Image> {
        match self {
            Self::IceMage => sprites.ice_mage_enemy_sprite_sheet.clone(),
            Self::Warrior => sprites.warrior_enemy_sprite_sheet.clone(),
            Self::FireMage => sprites.fire_mage_enemy_sprite_sheet.clone(),
        }
    }
}

pub fn spawn_enemies(
    enemy_trigger: Trigger<EnemiesSpawnEvent>,
    mut commands: Commands,
    enemy_assets: Res<EnemyAssets>,
    animation_config: Res<DefaultAnimationConfig>,
    sprites: Res<SpriteAssets>,
    atlases: Res<SpriteSheetLayouts>,
) {
    for spawn_data in enemy_trigger.0.clone() {
        let enemy_name = spawn_data.enemy_type.name();
        spawn_enemy(
            &mut commands,
            &enemy_name,
            &enemy_assets,
            spawn_data,
            &animation_config,
            &sprites,
            &atlases,
        );
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    enemy_name: &str,
    enemy_assets: &EnemyAssets,
    spawn_data: EnemySpawnData,
    animation_config: &DefaultAnimationConfig,
    sprites: &SpriteAssets,
    atlases: &SpriteSheetLayouts,
) {
    if let Some(enemy_details) = enemy_assets.enemy_config.get(enemy_name) {
        let sprite = Sprite::from_atlas_image(
            spawn_data.enemy_type.sprite(sprites),
            TextureAtlas {
                layout: atlases.enemy_atlas_layout.clone(),
                index: animation_config
                    .get_indices(ActionState::Idle, FacingDirection::Down)
                    .first,
            },
        );

        let starting_items = [
            spawn_mainhand_weapon(commands, sprites, atlases, &enemy_details.weapon),
            spawn_health_potion(commands, sprites),
        ];

        let enemy = commands
            .spawn((
                Enemy,
                Inventory::builder()
                    .items(starting_items.into())
                    .coins(99)
                    .max_capacity(10)
                    .build(),
                SimpleMotion::new(enemy_details.simple_motion_speed),
                Health::new(enemy_details.health),
                LockedAxes::new().lock_rotation(),
                RigidBody::Dynamic,
                AimPosition::default(),
                Mana::new(100.0, 10.0),
                ActionState::Idle,
                Collider::rectangle(enemy_details.collider_size.0, enemy_details.collider_size.1),
                CollisionLayers::new(
                    [GameCollisionLayer::Grounded, GameCollisionLayer::Enemy],
                    [
                        GameCollisionLayer::InAir,
                        GameCollisionLayer::Grounded,
                        GameCollisionLayer::HighObstacle,
                        GameCollisionLayer::LowObstacle,
                    ],
                ),
                (
                    Transform::from_translation(spawn_data.position),
                    animation_config.get_indices(ActionState::Idle, FacingDirection::Down),
                    AnimationTimer(
                        animation_config.get_timer(ActionState::Idle, FacingDirection::Down),
                    ),
                    sprite,
                    FacingDirection::Down,
                ),
            ))
            .add_children(&starting_items)
            .observe(on_enemy_defeated)
            .observe(on_equipment_activated)
            .id();

        commands
            .entity(starting_items[0])
            .insert(Equipped::new(enemy));
    } else {
        warn!("Enemy {} not found in enemy config.", enemy_name);
    }
}
