use avian2d::prelude::*;
use bevy::prelude::*;
use rand::{thread_rng, Rng};
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

#[derive(Debug)]
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

pub fn spawn_enemies(
    enemy_trigger: Trigger<EnemiesSpawnEvent>,
    mut commands: Commands,
    enemy_assets: Res<EnemyAssets>,
    animation_config: Res<DefaultAnimationConfig>,
    sprites: Res<SpriteAssets>,
    atlases: Res<SpriteSheetLayouts>,
) {
    let enemy_spawn_positions = enemy_trigger.0.clone();
    for spawn_position in enemy_spawn_positions {
        let enemy_list = ["FireMage", "Warrior", "IceMage"];
        let mut rng = thread_rng();
        let choice = rng.gen_range(0..enemy_list.len());
        let enemy_to_spawn = enemy_list[choice];
        spawn_enemy(
            &mut commands,
            enemy_to_spawn,
            &enemy_assets,
            spawn_position,
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
    spawn_position: Vec3,
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
        warn!("Spawning wep {:?}", enemy_type.weapon);
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
                    Transform::from_translation(spawn_position),
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
        eprintln!("Enemy {} not found in enemy config.", enemy_name);
    }
}
