//! What a brain knows: vision tracks a target and aggro picks one.

use std::f32::consts::FRAC_PI_4;

use avian2d::prelude::{RayCaster, RayHits};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::BrainSystems;
use crate::prelude::*;

/// Seconds a character keeps a target it can no longer see.
const TARGET_MEMORY_SECONDS: f32 = 3.0;

/// Distance within which a character notices the watched entity even outside its vision cone.
const NOTICE_DISTANCE: f32 = 3.0 * TILE_SIZE as f32;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_damage_aggro)
        .add_observer(on_targeting_added_record_time)
        .add_systems(
            Update,
            (
                (update_target_info, is_target_in_sight)
                    .chain()
                    .in_set(BrainSystems::Perception),
                (should_target_watched, should_stop_targeting).in_set(BrainSystems::Targeting),
                update_aim_position.in_set(BrainSystems::Aim),
            ),
        );
}

/// How wide an entity's vision cone is.
#[derive(Component)]
#[component(immutable)]
#[require(Vision, TargetInfo)]
pub(crate) struct VisionCone {
    /// In radians; `PI / 4.0` is a 90° cone.
    half_angle: f32,
}

impl Default for VisionCone {
    fn default() -> Self {
        Self {
            half_angle: FRAC_PI_4,
        }
    }
}

impl VisionCone {
    /// A zero-width cone, which never contains a direction.
    #[cfg(feature = "dev_tools")]
    pub(crate) fn blind() -> Self {
        Self { half_angle: 0.0 }
    }

    fn contains_direction(&self, aim_direction: Vec2, direction: Vec2) -> bool {
        direction.dot(aim_direction) > self.half_angle.cos()
    }
}

/// Stores calculated perception data for a given potential or current target. A target out of
/// sight is tracked at its last known position; see [`update_target_info`].
#[derive(Component, Default, Clone)]
pub(crate) struct TargetInfo {
    /// Distance to the observed entity.
    pub distance: f32,
    /// Direction vector pointing to the observed entity.
    pub(super) direction: Vec2,
    /// World position of the observed entity.
    pub position: Vec2,
    /// Intended world velocity of the observed entity.
    pub(super) velocity: Vec2,
    /// Whether the observed entity is within an unobstructed line of sight (based on `RayCaster`).
    pub line_of_sight: bool,
    /// Whether the observed entity is within the entity’s vision cone angle.
    pub(super) in_vision_cone: bool,
    /// When the current `Targeting` was acquired, in virtual seconds.
    pub(super) targeted_at: f32,
    /// When the target was last in sight or held by a `TargetLock`, in virtual seconds.
    last_seen_at: f32,
}

impl TargetInfo {
    /// Records where the observed entity is relative to `observer_position`, and how it moves.
    fn record_observation(
        &mut self,
        observer_position: Vec2,
        target_position: Vec2,
        target_velocity: Vec2,
    ) {
        self.direction = (target_position - observer_position).normalize_or_zero();
        self.distance = target_position.distance(observer_position);
        self.position = target_position;
        self.velocity = target_velocity;
    }

    pub(super) fn has_finite_distance(&self) -> bool {
        self.distance.is_finite()
    }

    /// Whether the observed entity is at a finite distance of at most `reach`.
    pub(super) fn is_within(&self, reach: f32) -> bool {
        self.has_finite_distance() && self.distance <= reach
    }

    /// The direction to the observed entity; `None` while the two overlap, leaving no
    /// direction to aim along.
    pub(super) fn nonzero_direction(&self) -> Option<Vec2> {
        (self.direction.length_squared() > f32::EPSILON).then_some(self.direction)
    }

    /// Whether the observed entity is inside the vision cone, ignoring walls.
    #[cfg(feature = "dev_native")]
    pub(crate) fn in_vision_cone(&self) -> bool {
        self.in_vision_cone
    }

    /// Whether the observed entity is in line of sight and either inside the vision cone or
    /// within [`NOTICE_DISTANCE`].
    fn is_noticed(&self) -> bool {
        self.line_of_sight && (self.in_vision_cone || self.is_within(NOTICE_DISTANCE))
    }
}

/// Finds a character's allies: the other living characters on its side, gathered on each call.
#[derive(SystemParam)]
pub(crate) struct Allies<'w, 's> {
    characters: Query<
        'w,
        's,
        (
            Entity,
            &'static Transform,
            &'static CombatSide,
            &'static Health,
        ),
    >,
}

/// An ally found by [`Allies`], and its distance from the character that asked.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ally {
    pub entity: Entity,
    pub distance: f32,
}

impl Allies<'_, '_> {
    /// The closest ally of `entity`, if any is alive.
    pub(crate) fn closest(&self, entity: Entity) -> Option<Ally> {
        self.allies_of(entity)
            .min_by(|ally, other| ally.distance.total_cmp(&other.distance))
    }

    /// The allies of `entity` at most `reach` away, in no particular order.
    pub(crate) fn within(&self, entity: Entity, reach: f32) -> impl Iterator<Item = Ally> {
        self.allies_of(entity)
            .filter(move |ally| ally.distance <= reach)
    }

    fn allies_of(&self, entity: Entity) -> impl Iterator<Item = Ally> {
        let caller = self.characters.get(entity).ok();
        caller
            .into_iter()
            .flat_map(move |(_, caller_transform, caller_side, _)| {
                let position = caller_transform.translation.xy();
                self.characters
                    .iter()
                    .filter(move |(ally, _, side, health)| {
                        *ally != entity && side == &caller_side && !health.is_dead()
                    })
                    .map(move |(ally, transform, ..)| Ally {
                        entity: ally,
                        distance: transform.translation.xy().distance(position),
                    })
            })
    }
}

/// The entity this one fights, added on sight or when damaged.
#[derive(Component)]
#[relationship(relationship_target = TargetedBy)]
pub(crate) struct Targeting(pub Entity);

/// Inverse of [`Targeting`].
#[derive(Component)]
#[relationship_target(relationship = Targeting)]
pub(crate) struct TargetedBy(Vec<Entity>);

/// The entity this one watches for, to target once sighted.
#[derive(Component, FromTemplate, Clone)]
#[relationship(relationship_target = WatchedBy)]
pub(in crate::character) struct Watching(pub Entity);

/// Inverse of [`Watching`].
#[derive(Component)]
#[relationship_target(relationship = Watching)]
pub(in crate::character) struct WatchedBy(Vec<Entity>);

/// Tracks the target live without sight until removed. Damage aggro schedules its removal.
#[derive(Component)]
pub(crate) struct TargetLock;

// ---------------------
// VISION + PERCEPTION
// ---------------------

/// Aims vision at the target while targeting, otherwise along the facing direction, as far out as
/// the tracked target.
fn update_aim_position(
    mut character_query: Query<
        (&mut Vision, &TargetInfo, Has<Targeting>, &FacingDirection),
        Without<Player>,
    >,
) {
    character_query.par_iter_mut().for_each(
        |(mut vision, target_info, has_target, facing_direction)| {
            vision.aim_direction = if has_target {
                target_info.direction
            } else {
                facing_direction.to_vec2()
            };
            vision.aim_distance = target_info.distance;
        },
    );
}

/// Points the sight ray at the target (or the watched entity without one) and tracks its
/// direction and distance. A target that is neither in sight nor held by a [`TargetLock`] stays
/// at its last known position, as if standing still there.
fn update_target_info(
    time: Res<Time<Virtual>>,
    mut npc_query: Query<
        (
            &mut TargetInfo,
            &mut RayCaster,
            &Transform,
            &Watching,
            Option<&Targeting>,
            Has<TargetLock>,
        ),
        Without<Player>,
    >,
    target_query: Query<(&Transform, Option<(&Kinematics, &StatMultipliers)>)>,
) {
    let now = time.elapsed_secs();
    npc_query.par_iter_mut().for_each(
        |(mut target_info, mut ray_caster, transform, watching, targeting, has_lock)| {
            let target_entity = targeting.map_or(watching.0, |t| t.0);
            let Ok((target_transform, target_movement)) = target_query.get(target_entity) else {
                return;
            };
            let position = transform.translation.xy();
            let target_position = target_transform.translation.xy();
            // Aimed at the feet, which a wall never covers, unlike the head of a character
            // standing under one.
            let target_feet = target_position + Vec2::Y * CHARACTER_FEET_POS_OFFSET;
            let ray_origin = position + ray_caster.origin;
            ray_caster.direction = Dir2::new(target_feet - ray_origin).unwrap_or(Dir2::X);

            let tracked_live = targeting.is_none() || target_info.line_of_sight || has_lock;
            if tracked_live {
                let target_velocity = target_movement
                    .map_or(Vec2::ZERO, |(kinematics, stat_multipliers)| {
                        kinematics.velocity(stat_multipliers.speed)
                    });
                target_info.last_seen_at = now;
                target_info.record_observation(position, target_position, target_velocity);
            } else {
                let last_known_position = target_info.position;
                target_info.record_observation(position, last_known_position, Vec2::ZERO);
            }
        },
    );
}

/// Updates line of sight (the ray hit the target or one of its children) and
/// whether the target is inside the vision cone.
fn is_target_in_sight(
    mut npc_query: Query<(
        &mut TargetInfo,
        &RayHits,
        &Vision,
        &VisionCone,
        &Watching,
        Option<&Targeting>,
    )>,
    target_query: Query<Option<&Children>, Or<(With<WatchedBy>, With<TargetedBy>)>>,
) {
    npc_query.par_iter_mut().for_each(
        |(mut target_info, ray_hits, vision, vision_cone, watching, targeting)| {
            target_info.in_vision_cone =
                vision_cone.contains_direction(vision.aim_direction, target_info.direction);

            let target_entity = targeting.map_or(watching.0, |t| t.0);

            let target_children = target_query.get(target_entity).ok().flatten();
            target_info.line_of_sight = ray_hits.iter().any(|hit| {
                hit.entity == target_entity
                    || target_children.is_some_and(|children| children.contains(&hit.entity))
            });
        },
    );
}

// ---------------------
// AGGRO / TARGETING
// ---------------------

const TARGET_LOCK_SECONDS: f32 = 6.0;

/// Targets the attacker on any hit, ignoring sight, or the watched entity for
/// attacker-less damage.
fn on_damage_aggro(
    damage_dealt: On<DamageDealt>,
    mut commands: Commands,
    target_query: Query<&Watching>,
) {
    let damaged_entity = damage_dealt.entity;

    let attacker_or_watched = damage_dealt.attacker.or_else(|| {
        target_query
            .get(damaged_entity)
            .ok()
            .map(|watching| watching.0)
    });

    if let Some(target) = attacker_or_watched {
        debug!("I've been hit: {damaged_entity}, attacking: {target}");
        lock_onto_target(&mut commands, damaged_entity, target);
    }
}

/// Makes `entity` target `target` and keep it without sight for [`TARGET_LOCK_SECONDS`].
pub(crate) fn lock_onto_target(commands: &mut Commands, entity: Entity, target: Entity) {
    commands
        .entity(entity)
        .insert((TargetLock, Targeting(target)));

    schedule_component_removal::<TargetLock>(commands, entity, TARGET_LOCK_SECONDS);
}

/// Records when targeting begins. Only on add, so re-inserts from repeated
/// hits and target switches keep the clock.
fn on_targeting_added_record_time(
    added: On<Add<Targeting>>,
    time: Res<Time<Virtual>>,
    mut target_infos: Query<&mut TargetInfo>,
) {
    if let Ok(mut target_info) = target_infos.get_mut(added.entity) {
        target_info.targeted_at = time.elapsed_secs();
        target_info.last_seen_at = time.elapsed_secs();
    }
}

/// Starts targeting the watched entity once it is noticed; see [`TargetInfo::is_noticed`].
fn should_target_watched(
    mut commands: Commands,
    npc_query: Query<(&TargetInfo, &Watching, Entity), Without<Targeting>>,
) {
    for (target_info, watching, entity) in &npc_query {
        if target_info.is_noticed() {
            commands.entity(entity).insert(Targeting(watching.0));
        }
    }
}

/// Stops targeting a target unseen for [`TARGET_MEMORY_SECONDS`].
fn should_stop_targeting(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    npc_query: Query<(&TargetInfo, Entity), With<Targeting>>,
) {
    let now = time.elapsed_secs();
    for (target_info, entity) in &npc_query {
        if now - target_info.last_seen_at > TARGET_MEMORY_SECONDS {
            commands.entity(entity).remove::<Targeting>();
        }
    }
}
