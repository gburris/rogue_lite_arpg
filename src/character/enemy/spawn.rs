use avian2d::prelude::{RayCaster, SpatialQueryFilter};
use bevy::prelude::*;
use bevy::ui_widgets::observe;

use super::{Enemy, EnemyKind, bat, defeat, mage, warrior};
use crate::character::brain::{Brain, HomeArea, Interval, Wander, Watching};
use crate::character::physical_collider;
use crate::prelude::*;

#[derive(Event, Debug)]
pub struct SpawnEnemies(pub Vec<EnemySpawnData>);

#[derive(Clone, Copy, Debug)]
pub struct EnemySpawnData {
    pub position: Vec2,
    pub enemy_kind: EnemyKind,
}

pub(super) fn spawn_enemies(
    spawn_enemies: On<SpawnEnemies>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
    player: Single<Entity, With<Player>>,
    mut map_rng: ResMut<MapRng>,
) {
    for &spawn_data in &spawn_enemies.0 {
        spawn_enemy(
            &mut commands,
            spawn_data,
            &sprites,
            &sprite_layouts,
            &shadows,
            player.entity(),
            &mut map_rng,
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
    map_rng: &mut MapRng,
) {
    trace!("Spawning enemy at: {}", spawn_data.position);

    let base = base_enemy(spawn_data.enemy_kind, spawn_data.position, player);
    match spawn_data.enemy_kind {
        EnemyKind::Warrior => {
            warrior::spawn_warrior(commands, base, sprites, sprite_layouts, shadows);
        }
        EnemyKind::IceMage => {
            mage::spawn_ice_mage(commands, base, sprites, sprite_layouts, shadows);
        }
        EnemyKind::FireMage => {
            mage::spawn_fire_mage(commands, base, sprites, sprite_layouts, shadows, map_rng);
        }
        EnemyKind::Bat => bat::spawn_bat(commands, base, sprites, sprite_layouts, shadows),
    }
}

pub(super) fn spawn_enemy_with_equipment(
    commands: &mut Commands,
    enemy: impl Bundle,
    mainhand: impl Bundle,
) -> Entity {
    let enemy = commands.spawn(enemy).id();

    let mainhand = commands.spawn(mainhand).id();

    commands.trigger(Equip {
        item: mainhand,
        holder: enemy,
    });

    enemy
}

/// Enemies stand a while between short walks while disengaged.
pub(super) fn enemy_wander() -> Wander {
    Wander::new(Interval::new(3.0, 5.0), Interval::new(1.0, 2.0))
}
const ENEMY_SIGHT_RANGE: f32 = 350.0;

/// The bundle every enemy kind shares; each kind's file adds the rest.
fn base_enemy(kind: EnemyKind, position: Vec2, player: Entity) -> impl Bundle {
    (
        Enemy,
        Brain::new(kind.brain()),
        Transform::from_translation(position.extend(ZLayer::OnGround.z())),
        HomeArea::new(position, 8.0 * TILE_SIZE as f32, TILE_SIZE as f32 / 2.0),
        Watching(player),
        observe(defeat::on_enemy_defeated),
    )
}

pub(super) fn sight_ray(range: f32, mask: SpatialQueryFilter) -> RayCaster {
    RayCaster::default()
        .with_max_distance(range)
        .with_query_filter(mask)
        .with_max_hits(1)
}

/// Sight for enemies on the ground, where high obstacles block the ray.
pub(super) fn ground_sight_ray() -> RayCaster {
    sight_ray(
        ENEMY_SIGHT_RANGE,
        SpatialQueryFilter::from_mask([
            GameCollisionLayer::AllyHurtBox,
            GameCollisionLayer::HighObstacle,
        ]),
    )
}

pub(super) fn enemy_children(shadows: &Shadows) -> impl Bundle {
    children![
        shadow(shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
        physical_collider(),
        hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::EnemyHurtBox),
    ]
}
