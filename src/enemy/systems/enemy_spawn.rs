use avian2d::prelude::*;
use bevy::prelude::*;
use serde::Serialize;

use crate::{
    animation::{AnimationTimer, DefaultAnimationConfig, FacingDirection},
    combat::{
        attributes::{Health, Mana},
        components::{ActionState, AimPosition},
    },
    configuration::{
        assets::{SpriteAssets, SpriteSheetLayouts},
        GameCollisionLayer,
    },
    enemy::{systems::on_enemy_defeated, Enemy, EnemyAssets},
    items::{
        equipment::{on_main_hand_activated, Equipped},
        inventory::Inventory,
        spawn_health_potion, spawn_mainhand_weapon,
    },
    map::EnemiesSpawnEvent,
    movement::components::SimpleMotion,
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

pub fn get_name_from_type(enemy_type: EnemyType) -> String {
    match enemy_type {
        EnemyType::IceMage => return "IceMage".to_owned(),
        EnemyType::Warrior => return "Warrior".to_owned(),
        EnemyType::FireMage => return "FireMage".to_owned(),
    };
}

pub fn spawn_enemies(
    enemy_trigger: Trigger<EnemiesSpawnEvent>,
    mut commands: Commands,
    enemy_assets: Res<EnemyAssets>,
    animation_config: Res<DefaultAnimationConfig>,
    sprites: Res<SpriteAssets>,
    atlases: Res<SpriteSheetLayouts>,
) {
    let enemies_spawn_data = enemy_trigger.0.clone();
    for spawn_data in enemies_spawn_data {
        let enemy_to_spawn = get_name_from_type(spawn_data.enemy_type);
        spawn_enemy(
            &mut commands,
            &enemy_to_spawn,
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
    enemy_assets: &Res<EnemyAssets>,
    spawn_data: EnemySpawnData,
    animation_config: &Res<DefaultAnimationConfig>,
    sprites: &Res<SpriteAssets>,
    atlases: &Res<SpriteSheetLayouts>,
) {
    if let Some(enemy_type) = enemy_assets.enemy_config.get(enemy_name) {
        let sprite = Sprite::from_atlas_image(
            sprites.enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: atlases.enemy_atlas_layout.clone(),
                index: animation_config
                    .get_indices(ActionState::Idle, FacingDirection::Down)
                    .first,
            },
        );
        let weapon = spawn_mainhand_weapon(commands, &sprites, &atlases, &enemy_type.weapon);
        let health_potion = spawn_health_potion(commands, &sprites);
        let starting_items = [weapon, health_potion];
        let enemy = commands
            .spawn((
                Enemy,
                Inventory::builder()
                    .items(starting_items.into())
                    .coins(99)
                    .max_capacity(10)
                    .build(),
                SimpleMotion::new(enemy_type.simple_motion_speed),
                Health::new(enemy_type.health),
                LockedAxes::new().lock_rotation(),
                RigidBody::Dynamic,
                AimPosition::default(),
                Mana::new(100.0, 10.0),
                ActionState::Idle,
                Collider::rectangle(enemy_type.collider_size.0, enemy_type.collider_size.1),
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
            .observe(on_main_hand_activated)
            .id();

        commands
            .entity(starting_items[0])
            .insert(Equipped::new(enemy));
    } else {
        warn!("Enemy {} not found in enemy config.", enemy_name);
    }
}
