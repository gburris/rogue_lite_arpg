use avian2d::prelude::*;
use bevy::prelude::*;
use serde::Serialize;

use crate::{
    ai::SimpleMotion,
    combat::{damage::HurtBox, Health, Mana},
    configuration::{
        assets::{Shadows, SpriteAssets, SpriteSheetLayouts},
        spawn_shadow, GameCollisionLayer, CHARACTER_FEET_POS_OFFSET,
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
    pub position: Vec2,
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
    sprites: Res<SpriteAssets>,
    atlases: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
) {
    for spawn_data in enemy_trigger.0.clone() {
        let enemy_name = spawn_data.enemy_type.name();
        spawn_enemy(
            &mut commands,
            &enemy_name,
            &enemy_assets,
            spawn_data,
            &sprites,
            &atlases,
            &shadows,
        );
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    enemy_name: &str,
    enemy_assets: &EnemyAssets,
    spawn_data: EnemySpawnData,
    sprites: &SpriteAssets,
    atlases: &SpriteSheetLayouts,
    shadows: &Shadows,
) {
    if let Some(enemy_details) = enemy_assets.enemy_config.get(enemy_name) {
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
                Mana::new(100.0, 10.0),
                Transform::from_translation(spawn_data.position.extend(0.0)),
                Sprite::from_atlas_image(
                    spawn_data.enemy_type.sprite(sprites),
                    TextureAtlas {
                        layout: atlases.enemy_atlas_layout.clone(),
                        ..default()
                    },
                ),
            ))
            .with_children(|spawner| {
                spawn_shadow(spawner, shadows, CHARACTER_FEET_POS_OFFSET - 4.0);

                // collider to bump into stuff
                spawner.spawn((
                    Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0),
                    Collider::circle(10.0),
                    CollisionLayers::new(
                        [GameCollisionLayer::Grounded],
                        [
                            GameCollisionLayer::Grounded,
                            GameCollisionLayer::HighObstacle,
                            GameCollisionLayer::LowObstacle,
                        ],
                    ),
                ));

                // hurtbox
                spawner.spawn((
                    HurtBox,
                    Collider::rectangle(
                        enemy_details.collider_size.0,
                        enemy_details.collider_size.1,
                    ),
                    Transform::from_xyz(0.0, -8.0, 0.0),
                    Sensor,
                    CollisionLayers::new(
                        [GameCollisionLayer::EnemyHurtBox],
                        [GameCollisionLayer::HitBox],
                    ),
                ));
            })
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
