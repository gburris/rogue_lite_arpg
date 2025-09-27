use avian2d::prelude::{RayCaster, SpatialQueryFilter};
use bevy::prelude::*;
use bevy_behave::prelude::*;
use bevy_bundled_observers::observers;
use serde::Serialize;

mod defeat;

use crate::{
    character::{
        behavior::{Anchor, AttemptMelee, Chase, Idle, KeepDistanceAndFire, Retreat, Wander},
        physical_collider,
        vision::{VisionCapabilities, Watching},
        Character,
    },
    combat::{damage::hurtbox, Health, Mana},
    configuration::{
        assets::{Shadows, SpriteAssets, SpriteSheetLayouts},
        shadow, GameCollisionLayer, CHARACTER_FEET_POS_OFFSET,
    },
    economy::Purse,
    items::{
        equipment::{on_equipment_activated, Equipped},
        fire_staff, health_potion,
        inventory::Inventory,
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

#[derive(Debug, Clone, Serialize, Component, Copy, PartialEq)]
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
    let starting_items = [
        commands.spawn(fire_staff(sprites, sprite_layouts)).id(),
        commands.spawn(health_potion(sprites)).id(),
    ];

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

    let enemy_behavior = if spawn_data.enemy_type == EnemyType::Warrior {
        melee_enemy_behavior
    } else {
        ranged_enemy_behavior
    };

    let enemy = commands
        .spawn((
            base_enemy(spawn_data.position, starting_items.into(), player),
            fire_mage(sprites, sprite_layouts),
            enemy_children(enemy_behavior, &shadows),
        ))
        .add_children(&starting_items)
        .id();

    commands
        .entity(starting_items[0])
        .insert(Equipped::new(enemy));
}

fn base_enemy(position: Vec2, starting_items: Vec<Entity>, player: Entity) -> impl Bundle {
    (
        Enemy,
        Transform::from_translation(position.extend(0.0)),
        Anchor::new(position, 256.0), // 8 tile radius
        Mana::new(100.0, 10.0),
        Inventory::builder()
            .items(starting_items)
            .max_capacity(10)
            .build(),
        // enemy vision distance
        RayCaster::default()
            .with_max_distance(350.0)
            .with_query_filter(SpatialQueryFilter::from_mask([
                GameCollisionLayer::AllyHurtBox,
                GameCollisionLayer::HighObstacle,
            ]))
            .with_max_hits(1),
        Watching(player),
        observers![defeat::on_enemy_defeated, on_equipment_activated],
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
    )
}
