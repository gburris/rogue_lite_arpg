use avian2d::prelude::{RayCaster, SpatialQueryFilter};
use bevy::prelude::*;
use super::{Enemy, bat, mage, warrior};
use crate::character::brain::considerations::{CAST_HYSTERESIS, CAST_MAX_RANGE};
use crate::character::brain::{Brain, Watching};
use crate::character::motion::{Chase, Home, HomeArea, ReturnToHome, Wander};
use crate::character::physical_collider;
use crate::interval::Interval;
use crate::prelude::*;
use serde::Deserialize;
use crate::character::brain::BrainDefinition;
use avian2d::prelude::RigidBody;
use rand::{RngExt, rng};
use crate::character::motion::MotionCommands;
use crate::character::player::PlayerAttributes;

#[derive(Event, Debug)]
pub struct SpawnEnemies(pub Vec<EnemySpawn>);

#[derive(Clone, Copy, Debug)]
pub struct EnemySpawn {
    pub position: Vec2,
    pub kind: EnemyKind,
}

pub(super) fn on_spawn_enemies(
    spawn_enemies: On<SpawnEnemies>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
    player: Single<Entity, With<Player>>,
    mut map_rng: ResMut<MapRng>,
) {
    let _span = info_span!("enemy::on_spawn_enemies").entered();
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
    spawn_data: EnemySpawn,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
    player: Entity,
    map_rng: &mut MapRng,
) {
    trace!("Spawning enemy at: {}", spawn_data.position);

    let base = base_enemy(spawn_data.kind, spawn_data.position, player);
    match spawn_data.kind {
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
    enemy: impl Scene,
    mainhand: impl Scene,
) -> Entity {
    let enemy = commands.spawn_scene(enemy).id();

    let mainhand = commands.spawn_scene(mainhand).id();

    commands.trigger(Equip {
        item: mainhand,
        holder: enemy,
    });

    enemy
}

/// The motion sources every enemy has: a wander that stands a while between short walks, a
/// return home and a chase. Each kind's file lists them first among its own.
pub(super) fn enemy_motion_sources() -> impl SceneList {
    bsn_list! {
        ~Wander::new(Interval::new(3.0, 5.0), Interval::new(1.0, 2.0))
        --
        ReturnToHome
        --
        Chase
    }
}

/// How far a ground enemy sees. Past the cast range, so a caster at its outer edge keeps its
/// target in sight.
const ENEMY_SIGHT_RANGE: f32 = 420.0;

const _: () = assert!(
    CAST_MAX_RANGE + CAST_HYSTERESIS < ENEMY_SIGHT_RANGE,
    "a caster holding the outer cast edge must still see its target"
);

/// The scene every enemy kind shares; each kind's file adds the rest.
fn base_enemy(kind: EnemyKind, position: Vec2, player: Entity) -> impl Scene {
    let brain = kind.brain();
    bsn! {
        Enemy
        template(move |_| Ok(kind))
        template(move |_| Ok(Brain::new(brain)))
        Transform::from_translation({position.extend(ZLayer::OnGround.z())})
        Home [
            Name::new("Home")
            ~HomeArea::new(8.0 * TILE_SIZE as f32, TILE_SIZE as f32 / 2.0)
            Transform::from_translation({position.extend(0.0)})
        ]
        Watching(player)
        on(on_enemy_defeated)
    }
}

/// A sight ray cast from `origin_height` above the character's transform. Cast it from the
/// height of the body collider, which walls keep clear of: a ray from the sprite's center
/// starts inside the wall the character stands under.
pub(super) fn sight_ray(range: f32, origin_height: f32, mask: SpatialQueryFilter) -> RayCaster {
    RayCaster::new(Vec2::new(0.0, origin_height), Dir2::X)
        .with_max_distance(range)
        .with_query_filter(mask)
        .with_max_hits(1)
}

/// Sight for enemies on the ground, where high obstacles block the ray.
pub(super) fn ground_sight_ray() -> RayCaster {
    sight_ray(
        ENEMY_SIGHT_RANGE,
        CHARACTER_FEET_POS_OFFSET,
        SpatialQueryFilter::from_mask([
            GameCollisionLayer::AllyHurtBox,
            GameCollisionLayer::HighObstacle,
        ]),
    )
}

/// The shadow, body collider and hurtbox under every walking enemy.
pub(super) fn enemy_children(shadows: &Shadows) -> impl Scene {
    bsn! {
        Children [
            @shadow(shadows, CHARACTER_FEET_POS_OFFSET - 4.0)
            --
            @physical_collider()
            --
            @hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::EnemyHurtBox)
        ]
    }
}

/// Recipes name these directly, e.g. `(what: Enemy(FireMage), ...)` in `map_recipes.ron`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum EnemyKind {
    Warrior,
    IceMage,
    FireMage,
    /// Flying enemy: circles its target and breathes fire without a weapon.
    Bat,
}

impl EnemyKind {
    /// Every enemy kind, used for random spawns and exhaustive iteration.
    pub const ALL: &[Self] = &[Self::Warrior, Self::IceMage, Self::FireMage, Self::Bat];

    pub fn brain(self) -> &'static BrainDefinition {
        match self {
            Self::Warrior => &warrior::WARRIOR,
            Self::IceMage => &mage::ICE_MAGE,
            Self::FireMage => &mage::FIRE_MAGE,
            Self::Bat => &bat::BAT,
        }
    }
}

/// Experience paid out when the player defeats this enemy. Scales with
/// threat: the tanky warrior pays most, the fragile flying bat least.
#[derive(Component, Default, Clone, Copy, Debug, Deref, DerefMut)]
pub struct Experience(pub f32);

fn on_enemy_defeated(
    defeated: On<Defeated>,
    mut commands: Commands,
    mut defeated_enemy_query: Query<
        (
            Option<&Experience>,
            &Transform,
            Option<&Items>,
            Option<&Purse>,
        ),
        With<Enemy>,
    >,
    player_query: Single<(&PlayerAttributes, &mut Player)>,
    item_query: Query<&Item>,
    mut motions: MotionCommands,
) {
    let mut rng = rng();

    if let Ok((experience_to_gain, transform, items, purse)) =
        defeated_enemy_query.get_mut(defeated.entity)
    {
        let (player_attributes, mut player) = player_query.into_inner();

        if let Some(experience_to_gain) = experience_to_gain {
            player.experience += **experience_to_gain;
        }

        if let Some(items) = items {
            for item_entity in items.iter() {
                // Enemies drop their items based on drop rate
                if let Ok(item_result) = item_query.get(item_entity)
                    && rng.random::<f32>() < item_result.drop_rate
                {
                    commands.trigger(DropItem { item: item_entity });
                }
            }
        }

        if let Some(purse) = purse {
            // Enemies drop their gold based on player luck
            let gold_drop_chance = 0.1 + player_attributes.luck as f32 / 100.0;
            if rng.random::<f32>() < gold_drop_chance {
                commands.trigger(DropGold {
                    location: transform.translation.truncate(),
                    amount: purse.amount,
                });
            }
        }

        // Interruption ends action uses and finite motions only; see `InterruptionState`.
        motions.stop(defeated.entity);
        commands
            .entity(defeated.entity)
            .insert(Lifespan::new(2.0))
            .remove::<(RigidBody, Brain)>()
            .despawn_related::<Children>();
    }
}
