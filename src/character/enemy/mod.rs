use avian2d::prelude::{Collider, CollisionLayers, RayCaster, SpatialQueryFilter};
use bevy::{prelude::*, ui_widgets::observe};

mod defeat;

use super::brain::{
    Burst, CAST_BAND, CAST_MIN_RANGE, CastPlant, Kite, ManaRecovery, Slide, SlotIntent, Strafe,
    Swoop, TurnRate, Wander, WantsToBreathe, WantsToHeal,
};
use super::{AiSet, animation::AnimationSheet};
use crate::{
    character::{
        Character, Purse,
        anchor::Anchor,
        physical_collider,
        vision::{Targeting, VisionCapabilities, Watching},
    },
    prelude::*,
    world::MapRng,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_enemies);
    app.add_systems(
        Update,
        (apply_bat_fire, apply_bat_bite).in_set(AiSet::Execute),
    );
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
    VisionCapabilities,
    Purse { amount: 50 },
    CombatSide = CombatSide::Enemy,
    DespawnOnExit::<AppState>(AppState::Playing)
)]
pub struct Enemy;

/// Recipes name these directly, e.g. `(what: Enemy(FireMage), ...)` in `map_recipes.ron`.
#[derive(Component, Clone, Debug, Eq, PartialEq, Hash, serde::Deserialize)]

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum EnemyKind {
    Warrior,
    IceMage,
    FireMage,
    /// Flying enemy: circles its target and breathes fire without a weapon.
    Bat,
}

impl EnemyKind {
    /// Every kind; a recipe's `RandomEnemy` picks uniformly from this list.
    pub const ALL: [Self; 3] = [Self::Warrior, Self::IceMage, Self::FireMage];

    /// Every enemy kind, used for random spawns and exhaustive iteration.
    pub const ALL: &[Self] = &[Self::Warrior, Self::IceMage, Self::FireMage, Self::Bat];

    /// Brain used by this enemy kind.
    pub fn brain_kind(self) -> BrainKind {
        match self {
            Self::Warrior => BrainKind::Warrior,
            Self::IceMage => BrainKind::IceMage,
            Self::FireMage => BrainKind::FireMage,
            Self::Bat => BrainKind::Bat,
        }
    }

    /// Base locomotion and health shared by every enemy kind. Casters add
    /// their own `Mana`; nothing else spends it, so the bat and warrior do not
    /// carry an unused pool.
    fn stats(self) -> impl Bundle {
        let (speed, max_health) = match self {
            Self::Warrior => (200.0, 40.0),
            Self::IceMage => (100.0, 20.0),
            Self::FireMage => (150.0, 20.0),
            Self::Bat => (170.0, 15.0),
        };
        (SimpleMotion::new(speed), Health::new(max_health))
    }

    /// Sight range and ray blocker mask for this kind.
    fn sight(self) -> (f32, SpatialQueryFilter) {
        match self {
            // Bats fly high: longer reach, and high obstacles never block the
            // ray, so a bat holds its target through cover.
            Self::Bat => (
                BAT_SIGHT_RANGE,
                SpatialQueryFilter::from_mask([GameCollisionLayer::AllyHurtBox]),
            ),
            _ => (
                ENEMY_SIGHT_RANGE,
                SpatialQueryFilter::from_mask([
                    GameCollisionLayer::AllyHurtBox,
                    GameCollisionLayer::HighObstacle,
                ]),
            ),
        }
    }

    /// Y-sort anchor for this kind's sprite sheet.
    fn feet_offset(self) -> f32 {
        match self {
            Self::Bat => BAT_FEET_POS_OFFSET,
            _ => CHARACTER_FEET_POS_OFFSET,
        }
    }

    /// Experience granted to the player for defeating this kind. Scales with
    /// threat: the tanky warrior pays most, the fragile flying bat least.
    fn experience(self) -> f32 {
        match self {
            Self::Warrior => 20.0,
            Self::IceMage => 12.0,
            Self::FireMage => 15.0,
            Self::Bat => 6.0,
        }
    }
}

/// Experience paid out when the player defeats this enemy.
#[derive(Component)]
pub struct Experience {
    pub base_exp: f32,
}

fn spawn_enemies(
    spawn_enemies: On<SpawnEnemies>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
    player: Single<Entity, With<Player>>,
    mut map_rng: ResMut<MapRng>,
) {
    for spawn_data in spawn_enemies.0.clone() {
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

    let kind = spawn_data.enemy_kind;
    match kind {
        EnemyKind::Warrior => {
            // Warriors drink the potion they already carry in their inventory
            // once their flee opens safe distance.
            spawn_enemy_with_equipment(
                commands,
                (
                    warrior(sprites, sprite_layouts),
                    base_enemy(spawn_data.position, player, kind),
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
                    base_enemy(spawn_data.position, player, kind),
                    enemy_children(shadows),
                ),
                ice_staff(sprites, sprite_layouts),
            );
        }

        EnemyKind::FireMage => {
            let enemy = (
                fire_mage(sprites, sprite_layouts),
                base_enemy(spawn_data.position, player, kind),
                enemy_children(shadows),
            );

            // Fire mages carry one of two flame staves, chosen evenly at spawn
            // from the seeded map stream so the variant replays per seed.
            if map_rng.flip() {
                spawn_enemy_with_equipment(
                    commands,
                    enemy,
                    mage_fire_staff_single_bolt(sprites, sprite_layouts),
                );
            } else {
                spawn_enemy_with_equipment(
                    commands,
                    enemy,
                    mage_fire_staff_fan(sprites, sprite_layouts),
                );
            }
        }

        EnemyKind::Bat => {
            // Bats carry no staff: the breath template rides on the bat
            // itself and `apply_bat_fire` launches it from brain intent.
            commands.spawn((
                bat(sprites, sprite_layouts),
                base_enemy(spawn_data.position, player, kind),
                bat_children(shadows),
            ));
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

const ENEMY_SIGHT_RANGE: f32 = 350.0;
const BAT_SIGHT_RANGE: f32 = 550.0;

fn base_enemy(position: Vec2, player: Entity, kind: EnemyKind) -> impl Bundle {
    (
        Enemy,
        Experience {
            base_exp: kind.experience(),
        },
        Transform::from_translation(position.extend(ZLayer::OnGround.z())),
        Anchor::new(position, 8.0 * TILE_SIZE as f32),
        kind.brain_kind(),
        sight_ray(kind),
        YSort::from_offset(kind.feet_offset()),
        Watching(player),
        observe(defeat::on_enemy_defeated),
    )
}

fn sight_ray(kind: EnemyKind) -> RayCaster {
    let (range, mask) = kind.sight();
    RayCaster::default()
        .with_max_distance(range)
        .with_query_filter(mask)
        .with_max_hits(1)
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
        EnemyKind::Warrior.stats(),
        // 80% of swings end in a break for a rolled 6-8 tile run inside a 90°
        // cone away from the target; the other 20% swing again.
        Kite::new(0.8, 6.0 * TILE_SIZE as f32, 8.0 * TILE_SIZE as f32),
        Wander::new(1.0, 2.0),
        SlotIntent::default(),
        WantsToHeal::default(),
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
        EnemyKind::IceMage.stats(),
        // Casters are the only kinds that spend mana.
        Mana::new(100.0, 10.0),
        // Ice bolts cost 15, so a mana break has to last long enough to matter.
        ManaRecovery::new(0.4, 0.75),
        // Three to four tiles of straight escape once the player crosses the
        // danger ring; the cooldown is the window to close again.
        Slide::new(
            3.0 * TILE_SIZE as f32,
            4.0 * TILE_SIZE as f32,
            2.2,
            CAST_MIN_RANGE - 30.0,
            1.2,
        ),
        // Stand through the 0.3 s ice bolt so the wind-up is readable.
        CastPlant::new(0.3),
        Wander::new(1.0, 2.0),
        SlotIntent::default(),
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
        EnemyKind::FireMage.stats(),
        // Casters are the only kinds that spend mana.
        Mana::new(100.0, 10.0),
        // Fire volleys are cheap: a short break is enough to refill.
        ManaRecovery::new(0.2, 0.45),
        // Two to three tiles of tangent hop after every volley. Inside the
        // close ring a pinned target gets a step-in spread instead.
        Burst::new(
            2.0 * TILE_SIZE as f32,
            3.0 * TILE_SIZE as f32,
            2.2,
            CAST_MIN_RANGE - 10.0,
        ),
        // Stand through the 0.3 s fire volley so the wind-up is readable.
        CastPlant::new(0.3),
        Strafe::smart(CAST_BAND),
        Wander::new(1.0, 2.0),
        SlotIntent::default(),
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

/// Flying enemy marker. Bats collide as airborne, passing over low obstacles.
#[derive(Component, Default)]
pub(crate) struct Bat;

const BAT_BREATH_PERIOD: f32 = 1.1;
const BAT_TURN_RATE: f32 = 6.0;
/// Half-width of a bat's rerolled orbit-radius offset from the band's centre.
const BAT_ORBIT_STANDOFF: f32 = 50.0;
/// Contact range for a bite in world units. Anatomy, not tuning.
const BITE_REACH: f32 = 30.0;
/// Bat sprites sort and clip at their own 32x32 frame edge.
const BAT_FEET_POS_OFFSET: f32 = -16.0;
const BAT_SHADOW_DROP: f32 = 4.0;
const BAT_BODY_OFFSET: f32 = BAT_FEET_POS_OFFSET + 8.0;
const BAT_HURTBOX_OFFSET: f32 = BAT_FEET_POS_OFFSET + 12.0;

/// Breath cooldown for the innate attack, ticked every frame and reset on fire.
#[derive(Component, Deref, DerefMut)]
struct BreathCooldown(Timer);

fn bat(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        EnemyKind::Bat.stats(),
        Bat,
        TurnRate(BAT_TURN_RATE),
        Strafe::with_standoff(CAST_BAND, BAT_ORBIT_STANDOFF),
        Wander::new(1.0, 2.0),
        WantsToBreathe::default(),
        // 35% chance per roll; after a dive ends, wait 2 s before the next
        // roll. The 6 dmg bite rides a twice-as-fast pass that flies through
        // the target and back out to roughly the circling radius.
        Swoop::new(0.35, 2.0, 6.0),
        Sprite::from_atlas_image(
            sprites.bat_enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.bat_enemy_layout.clone(),
                index: 0, // Down row; FacingDirection defaults to Down.
            },
        ),
        AnimationSheet::bat(),
        BreathCooldown(Timer::from_seconds(BAT_BREATH_PERIOD, TimerMode::Once)),
        related!(Projectiles[fireball(sprites, sprite_layouts, 0.0)]),
    )
}

fn bat_children(shadows: &Shadows) -> impl Bundle {
    children![
        shadow(shadows, BAT_FEET_POS_OFFSET - BAT_SHADOW_DROP),
        bat_collider(),
        hurtbox_at(
            Vec2::new(24.0, 24.0),
            GameCollisionLayer::EnemyHurtBox,
            BAT_HURTBOX_OFFSET,
        ),
    ]
}

fn bat_collider() -> impl Bundle {
    (
        Transform::from_xyz(0.0, BAT_BODY_OFFSET, 0.0),
        Collider::circle(8.0),
        CollisionLayers::new(
            [GameCollisionLayer::InAir],
            [GameCollisionLayer::HighObstacle],
        ),
    )
}

/// Launches the bat's innate fireball whenever the brain wants to breathe at a
/// sighted target. No staff or equipment is involved.
fn apply_bat_fire(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut bats: Query<
        (
            Entity,
            &WantsToBreathe,
            Has<Targeting>,
            &TargetInfo,
            &Health,
            &Transform,
            &CombatSide,
            &Projectiles,
            &mut BreathCooldown,
        ),
        With<Bat>,
    >,
) {
    for (
        entity,
        wants_breathe,
        has_target,
        target_info,
        health,
        transform,
        side,
        templates,
        mut cooldown,
    ) in &mut bats
    {
        // The defeated body lingers for its dying beat; the breath stops at 0 hp.
        if health.hp <= 0.0 {
            continue;
        }
        cooldown.tick(time.delta());
        if !(has_target && target_info.line_of_sight && wants_breathe.0) {
            continue;
        }
        if !cooldown.is_finished() {
            continue;
        }

        // A bat can sit exactly on the player, where the bearing collapses to
        // zero. There is no angle to fire along, so wait for separation
        // instead of launching a NaN projectile.
        let aim_direction = target_info.direction;
        if aim_direction.length_squared() <= f32::EPSILON {
            continue;
        }

        for template in templates.iter() {
            commands.trigger(FireProjectile {
                item: template,
                side: *side,
                holder: Some(entity),
                position: transform.translation.xy(),
                aim_direction,
            });
        }
        cooldown.reset();
    }
}
/// Lands a swooping bat's bite once per pass, healing the bat by the bite's
/// damage; the dive ends past the target and back out, or on a timeout.
///
/// Commitment ownership: `Decision` arms a dive and clears one that lost its
/// mode; the motion driver advances the pass; this applier ends a pass that
/// physically completed or timed out. Appliers never start a commitment or
/// change the mode.
fn apply_bat_bite(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut bats: Query<
        (
            Entity,
            &Transform,
            &mut Swoop,
            Option<&Targeting>,
            &TargetInfo,
            &Health,
        ),
        With<Bat>,
    >,
) {
    for (entity, transform, mut swoop, targeting, target_info, health) in &mut bats {
        if !swoop.is_active() || health.hp <= 0.0 {
            continue;
        }
        // No live target: the arbiter ends the dive on the next decision tick,
        // so there is nothing to land against here.
        let Some(targeting) = targeting else {
            continue;
        };
        let position = transform.translation.xy();
        // One hit per pass: flying through the target keeps the dive going,
        // but it cannot tick damage every frame of contact.
        if target_info.distance <= BITE_REACH && swoop.bite() {
            commands.trigger(AttemptDamage {
                entity: targeting.0,
                damage: Damage::Single(swoop.damage),
                source_item: None,
                attacker: Some(entity),
                direction: Some(target_info.direction),
                ..default()
            });
            // A landed bite feeds the bat. Skip the event at full health so
            // the heal overlay does not pop a zero.
            if health.hp < health.max_hp {
                commands.trigger(AttemptHeal {
                    entity,
                    amount: swoop.damage,
                });
            }
        }
        let now = time.elapsed_secs();
        if swoop.cleared(position, target_info.distance) || swoop.timed_out(now) {
            swoop.finish(now);
        }
    }
}
