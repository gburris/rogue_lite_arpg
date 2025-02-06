use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::{AnimationTimer, DefaultAnimationConfig, DefaultAnimations, MovementDirection},
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
        equipment::{use_equipped::on_main_hand_activated, EquipmentSlots},
        spawn_random_mainhand_weapon,
    },
    map::systems::instance::spawn_instance_entities::EnemySpawnEvent,
    movement::components::SimpleMotion,
};

pub fn spawn_enemies(
    enemy_trigger: Trigger<EnemySpawnEvent>,
    mut commands: Commands,
    enemy_assets: Res<EnemyAssets>,
    animation_config: Res<DefaultAnimationConfig>,
    sprites: Res<SpriteAssets>,
    atlases: Res<SpriteSheetLayouts>,
) {
    let enemy_spawn_positions = enemy_trigger.0.clone();
    for spawn_position in enemy_spawn_positions {
        spawn_enemy(
            &mut commands,
            "Merman",
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
    let random_mainhand: Entity = spawn_random_mainhand_weapon(commands, &sprites, &atlases);
    let sprite = Sprite::from_atlas_image(
        sprites.enemy_sprite_sheet.clone(),
        TextureAtlas {
            layout: atlases.enemy_atlas_layout.clone(),
            index: animation_config
                .get_indices(&DefaultAnimations::IdleDown)
                .first,
        },
    );
    if let Some(enemy) = enemy_assets.enemy_config.get(enemy_name) {
        commands
            .spawn((
                Enemy,
                SimpleMotion::new(enemy.simple_motion_speed),
                Health::new(enemy.health),
                LockedAxes::new().lock_rotation(),
                RigidBody::Dynamic,
                AimPosition::default(),
                Mana::new(100.0, 10.0),
                ActionState::None,
                EquipmentSlots {
                    mainhand: Some(random_mainhand),
                    head: None,
                },
                Collider::rectangle(enemy.collider_size.0, enemy.collider_size.1),
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
                    animation_config.get_indices(&DefaultAnimations::IdleDown),
                    AnimationTimer(animation_config.get_timer(&DefaultAnimations::IdleDown)),
                    sprite,
                    DefaultAnimations::IdleDown,
                    MovementDirection::None,
                ),
            ))
            .observe(on_enemy_defeated)
            .observe(on_main_hand_activated);
    } else {
        eprintln!("Enemy {} not found in enemy config.", enemy_name);
    }
}
