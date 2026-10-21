use avian2d::prelude::{RayCaster, SpatialQueryFilter};
use bevy::{prelude::*, ui_widgets::observe};

mod defeat;

use crate::{
    character::{
        Character, Purse,
        anchor::Anchor,
        physical_collider,
        vision::{VisionCapabilities, Watching},
    },
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_enemies);
}

#[derive(Debug, Event)]
pub struct SpawnEnemies(pub Vec<EnemySpawnData>);

#[derive(Debug, Clone)]
pub struct EnemySpawnData {
    pub position: Vec2,
    pub enemy_kind: EnemyKind,
}

#[derive(Component)]
#[require(
    Character,
    Experience,
    VisionCapabilities,
    Purse { amount: 50 },
    CombatSide = CombatSide::Enemy,
    DespawnOnExit::<AppState>(AppState::Playing)
)]
pub struct Enemy;

#[derive(Component, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EnemyKind {
    Warrior,
    IceMage,
    FireMage,
}

impl EnemyKind {
    /// Brain used by this enemy kind.
    pub fn brain_kind(&self) -> BrainKind {
        match self {
            Self::Warrior => BrainKind::Warrior,
            Self::IceMage | Self::FireMage => BrainKind::RangedMage,
        }
    }
}

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

fn spawn_enemies(
    spawn_enemies: On<SpawnEnemies>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
    player: Single<Entity, With<Player>>,
) {
    for spawn_data in spawn_enemies.0.clone() {
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
    trace!("Spawning enemy at: {}", spawn_data.position);

    let brain = spawn_data.enemy_kind.brain_kind();
    match spawn_data.enemy_kind {
        EnemyKind::Warrior => {
            // Warriors drink the potion they already carry in their inventory
            // once their flee opens safe distance.
            spawn_enemy_with_equipment(
                commands,
                (
                    warrior(sprites, sprite_layouts),
                    base_enemy(spawn_data.position, player, brain),
                    enemy_children(shadows),
                ),
                sword(sprites),
            );
        }

        EnemyKind::IceMage => {
            spawn_enemy_with_equipment(
                commands,
                (
                    ice_mage(sprites, sprite_layouts),
                    base_enemy(spawn_data.position, player, brain),
                    enemy_children(shadows),
                ),
                ice_staff(sprites, sprite_layouts),
            );
        }

        EnemyKind::FireMage => {
            spawn_enemy_with_equipment(
                commands,
                (
                    fire_mage(sprites, sprite_layouts),
                    base_enemy(spawn_data.position, player, brain),
                    enemy_children(shadows),
                ),
                fire_staff(sprites, sprite_layouts),
            );
        }
    }
}

fn spawn_enemy_with_equipment(
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

fn base_enemy(position: Vec2, player: Entity, brain: BrainKind) -> impl Bundle {
    (
        Enemy,
        Transform::from_translation(position.extend(ZLayer::OnGround.z())),
        Anchor::new(position, 8.0 * TILE_SIZE as f32),
        brain,
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
    )
}

fn enemy_children(shadows: &Shadows) -> impl Bundle {
    children![
        shadow(shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
        physical_collider(),
        hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::EnemyHurtBox),
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
        related!(Items[health_potion(sprites)]),
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
        related!(Items[health_potion(sprites)]),
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
        related!(Items[health_potion(sprites)]),
    )
}
