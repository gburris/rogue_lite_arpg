use avian2d::prelude::{RayCaster, SpatialQueryFilter};
use bevy::prelude::*;
use bevy_behave::prelude::*;
use data_loader::EnemyAssets;
use serde::Serialize;

mod brain;
mod data_loader;
mod defeat;

use crate::{
    character::{
        behavior::{Chase, Idle, Retreat},
        physical_collider, Character,
    },
    combat::{damage::hurtbox, Health, Mana},
    configuration::{
        assets::{Shadows, SpriteAssets, SpriteSheetLayouts},
        shadow, GameCollisionLayer, CHARACTER_FEET_POS_OFFSET,
    },
    items::{
        equipment::{on_equipment_activated, Equipped},
        inventory::Inventory,
        spawn_health_potion, spawn_mainhand_weapon,
    },
    labels::sets::InGameSet,
    map::EnemiesSpawnEvent,
    prelude::*,
};

use super::{
    behavior::{Anchor, Wander},
    Agro,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, data_loader::setup_enemy_assets)
            .add_observer(spawn_enemies)
            .add_systems(
                Update,
                (
                    brain::point_raycast_to_player,
                    brain::update_aim_position,
                    brain::should_agro_player,
                    brain::tick_agro_target_lock,
                    brain::is_player_in_sight,
                )
                    .chain()
                    .in_set(InGameSet::Simulation),
            );
    }
}

#[derive(Component)]
#[require(Character, Experience, Agro)]
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

fn spawn_enemies(
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

        let enemy_behavior = behave! {
            Behave::Forever => {
                Behave::Fallback => {
                    Behave::Sequence => {
                        Behave::spawn_named("Wander", Wander::builder().timer_range(1.0..2.0)),
                        Behave::spawn_named("Idle", Idle::default().timer_range(3.0..5.0)),
                    },
                    Behave::spawn_named("Retreat", Retreat),
                    Behave::spawn_named("Chase", Chase),
                }
            }
        };

        let enemy = commands
            .spawn((
                Enemy,
                Anchor::new(spawn_data.position, 256.0), // 8 tile radius
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
                // enemy vision distance
                RayCaster::default()
                    .with_max_distance(350.0)
                    .with_query_filter(SpatialQueryFilter::from_mask([
                        GameCollisionLayer::AllyHurtBox,
                        GameCollisionLayer::HighObstacle,
                    ]))
                    .with_max_hits(1),
                children![
                    shadow(&shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
                    physical_collider(),
                    hurtbox(
                        enemy_details.collider_size.into(),
                        GameCollisionLayer::EnemyHurtBox
                    ),
                    BehaveTree::new(enemy_behavior.clone()),
                ],
            ))
            .add_children(&starting_items)
            .observe(defeat::on_enemy_defeated)
            .observe(on_equipment_activated)
            .observe(brain::on_damage_agro)
            .id();

        commands
            .entity(starting_items[0])
            .insert(Equipped::new(enemy));
    } else {
        warn!("Enemy {} not found in enemy config.", enemy_name);
    }
}
