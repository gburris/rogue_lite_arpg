use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::{AnimationTimer, DefaultAnimationConfig, DefaultAnimations, MovementDirection},
    combat::attributes::Health,
    configuration::{
        assets::{SpriteAssets, SpriteSheetLayouts},
        GameCollisionLayer,
    },
    enemy::{systems::on_enemy_defeated, Enemy, EnemyAssets},
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
    if let Some(enemy) = enemy_assets.enemy_config.get(enemy_name) {
        let new_enemy = commands
            .spawn((
                Enemy,
                SimpleMotion::new(enemy.simple_motion_speed),
                Health::new(enemy.health),
                LockedAxes::new().lock_rotation(),
                RigidBody::Dynamic,
                Collider::rectangle(enemy.collider_size.0, enemy.collider_size.1),
                CollisionLayers::new(
                    [GameCollisionLayer::Enemy, GameCollisionLayer::Grounded],
                    [
                        GameCollisionLayer::Player,
                        GameCollisionLayer::InAir,
                        GameCollisionLayer::Grounded,
                        GameCollisionLayer::HighObstacle,
                        GameCollisionLayer::LowObstacle,
                    ],
                ),
                Transform::from_translation(spawn_position),
            ))
            .observe(on_enemy_defeated)
            .id();
        let sprite = Sprite::from_atlas_image(
            sprites.enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: atlases.enemy_atlas_layout.clone(),
                index: animation_config
                    .get_indices(&DefaultAnimations::IdleDown)
                    .first,
            },
        );

        //Todo move to setup file / trigger / something besides bloating this
        commands.entity(new_enemy).insert((
            animation_config.get_indices(&DefaultAnimations::IdleDown),
            AnimationTimer(animation_config.get_timer(&DefaultAnimations::IdleDown)),
            sprite,
            DefaultAnimations::IdleDown,
            MovementDirection::None,
        ));
    } else {
        eprintln!("Enemy {} not found in enemy config.", enemy_name);
    }
}
