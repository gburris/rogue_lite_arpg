//! What a brain knows: vision tracks a target, and [`DecisionInputs::read`] answers each
//! consideration's input straight from the world when scoring asks for it.

use std::f32::consts::FRAC_PI_4;

use avian2d::prelude::{RayCaster, RayHits};
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::BrainSystems;
use super::action::{Ability, Action, find_carried_potion};
use super::capability::{
    CapabilityProfiles, MotionActionKey, MotionActionTable, UseEligibilityTable,
};
use super::motion::{HomeArea, Motion};
use crate::prelude::*;

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

/// The brain's own earlier choices, as scoring reads them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct BrainMemory {
    pub motion: Option<Motion>,
    /// An attack finished since the last decision.
    pub attack_finished: bool,
}

/// The raw fact a consideration reads; see [`DecisionInputs::read`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConsiderationInput {
    HasTarget,
    LineOfSight,
    TargetDistance,
    MissingHealth,
    ManaFraction,
    /// Whether this motion's kind is the one running; its payload is ignored.
    MotionRunning(Motion),
    /// Whether the running motion holds the target at a distance.
    Circling,
    /// How far past its home area range the character is.
    HomeAreaPull,
    AttackFinished,
    /// Target distance over this motion's reach; at most `1.0` is in reach.
    MotionReachFraction(Motion),
    /// Target distance over this action's reach; at most `1.0` is in reach.
    ActionReachFraction(Action),
    SecondsSinceTargeted,
}

/// How wide an entity's vision cone is.
#[derive(Component)]
#[component(immutable)]
#[require(Vision, TargetInfo)]
pub(crate) struct VisionCone {
    /// In radians; `PI / 4.0` is a 90° cone.
    pub half_angle: f32,
}

impl Default for VisionCone {
    fn default() -> Self {
        Self {
            half_angle: FRAC_PI_4,
        }
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

/// Keeps `Targeting` without sight until removed. Damage aggro schedules its removal.
#[derive(Component)]
pub(crate) struct TargetLock;

// ---------------------
// VISION + PERCEPTION
// ---------------------

/// Aims vision at the target while targeting, otherwise along the facing direction.
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
        },
    );
}

/// Tracks direction and distance to the target (or the watched entity without
/// one) and points the sight ray there.
fn update_target_info(
    mut npc_query: Query<
        (
            &mut TargetInfo,
            &mut RayCaster,
            &Transform,
            &Watching,
            Option<&Targeting>,
        ),
        Without<Player>,
    >,
    target_query: Query<(&Transform, Option<&Kinematics>)>,
) {
    npc_query.par_iter_mut().for_each(
        |(mut target_info, mut ray_caster, transform, watching, targeting)| {
            let target_entity = targeting.map_or(watching.0, |t| t.0);

            if let Ok((target_transform, target_kinematics)) = target_query.get(target_entity) {
                let target_position = target_transform.translation.xy();
                let position = transform.translation.xy();

                let target_direction = (target_position - position).normalize_or_zero();

                target_info.direction = target_direction;
                target_info.distance = target_position.distance(position);
                target_info.position = target_position;
                target_info.velocity = target_kinematics.map_or(Vec2::ZERO, Kinematics::velocity);

                ray_caster.direction = Dir2::new(target_direction).unwrap_or(Dir2::X);
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
            let vision_cone_dot = vision_cone.half_angle.cos();
            target_info.in_vision_cone =
                target_info.direction.dot(vision.aim_direction) > vision_cone_dot;

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
        commands
            .entity(damaged_entity)
            .insert((TargetLock, Targeting(target)));

        schedule_component_removal::<TargetLock>(&mut commands, damaged_entity, 6.0);
    }
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
    }
}

/// Starts targeting the watched entity if it is in sight and in the vision cone.
fn should_target_watched(
    mut commands: Commands,
    npc_query: Query<(&TargetInfo, &Watching, Entity), Without<Targeting>>,
) {
    for (target_info, watching, entity) in &npc_query {
        if target_info.line_of_sight && target_info.in_vision_cone {
            commands.entity(entity).insert(Targeting(watching.0));
        }
    }
}

/// Stops targeting if the entity loses sight of the target or the target lock has expired.
fn should_stop_targeting(
    mut commands: Commands,
    npc_query: Query<(&TargetInfo, Has<TargetLock>, Entity), With<Targeting>>,
) {
    for (target_info, has_lock, entity) in &npc_query {
        if !target_info.line_of_sight && !has_lock {
            commands.entity(entity).remove::<Targeting>();
        }
    }
}

/// The queries behind each capability's world-allows-use check, shared by
/// [`DecisionInputs::use_eligibility`] and the appliers.
#[derive(SystemParam)]
pub(super) struct UseEligibilityQueries<'w, 's> {
    pub equipment: EquipmentRead<'w, 's>,
    pub consumables: Query<'w, 's, (), With<Consumable>>,
    pub items_in_use: Query<'w, 's, (), With<EquipmentInUse>>,
}

/// Whether the world lets each motion or action run now: usable equipment (or
/// a held use already running), and a carried potion for `Heal`.
fn build_use_eligibility_table(
    entity: Entity,
    items: Option<&Items>,
    use_eligibility_queries: &UseEligibilityQueries,
) -> UseEligibilityTable {
    let mut use_eligibility = MotionActionTable::splat(true);
    for slot in EquipmentSlot::ALL {
        let held_item_in_use = !slot.is_instant_use()
            && use_eligibility_queries
                .equipment
                .item_in_slot(entity, slot)
                .is_some_and(|item| use_eligibility_queries.items_in_use.contains(item));
        use_eligibility[Action::Use(slot)] = held_item_in_use
            || use_eligibility_queries
                .equipment
                .can_use(entity, slot)
                .is_ok();
    }
    use_eligibility[Ability::Heal] =
        find_carried_potion(items, &use_eligibility_queries.consumables).is_some();
    UseEligibilityTable(use_eligibility)
}

/// The components one decision reads its inputs from.
#[derive(QueryData)]
pub(super) struct DecisionInputComponents {
    entity: Entity,
    transform: &'static Transform,
    has_target: Has<Targeting>,
    target_info: Option<&'static TargetInfo>,
    health: Option<&'static Health>,
    mana: Option<&'static Mana>,
    home_area: Option<&'static HomeArea>,
    kinematics: Option<&'static Kinematics>,
    items: Option<&'static Items>,
}

/// One decision's view of the world: references only, read on demand while scoring.
#[derive(Clone, Copy)]
pub(super) struct DecisionInputs<'a, 'w, 's> {
    pub item: &'a DecisionInputComponentsItem<'w, 's>,
    pub profiles: &'a CapabilityProfiles,
    pub now: f32,
}

impl DecisionInputs<'_, '_, '_> {
    /// The tracked target while targeting one at a finite distance. `TargetInfo`
    /// also tracks a watched entity, so every target input reads through here.
    pub fn target(&self) -> Option<&TargetInfo> {
        self.item
            .target_info
            .filter(|info| self.item.has_target && info.distance.is_finite())
    }

    pub fn stunned(&self) -> bool {
        self.item.kinematics.is_some_and(Kinematics::is_stunned)
    }

    /// Whether the world lets each motion or action run now.
    pub fn use_eligibility(&self, queries: &UseEligibilityQueries) -> UseEligibilityTable {
        build_use_eligibility_table(self.item.entity, self.item.items, queries)
    }

    /// Answers `input`; `None` when it is absent, which vetoes the behavior.
    pub fn read(&self, input: ConsiderationInput, memory: BrainMemory) -> Option<f32> {
        let flag = |set: bool| Some(if set { 1.0 } else { 0.0 });
        match input {
            ConsiderationInput::HasTarget => flag(self.target().is_some()),
            ConsiderationInput::LineOfSight => self.target().and_then(|t| flag(t.line_of_sight)),
            ConsiderationInput::TargetDistance => self.target().map(|target| target.distance),
            ConsiderationInput::MissingHealth => {
                Some(1.0 - self.item.health.and_then(Health::fraction).unwrap_or(1.0))
            }
            ConsiderationInput::ManaFraction => {
                Some(self.item.mana.and_then(Mana::fraction).unwrap_or(1.0))
            }
            ConsiderationInput::MotionRunning(motion) => flag(
                memory
                    .motion
                    .is_some_and(|running| running.index() == motion.index()),
            ),
            ConsiderationInput::Circling => flag(matches!(
                memory.motion,
                Some(Motion::Orbit(_) | Motion::KeepDistance(_))
            )),
            ConsiderationInput::HomeAreaPull => self.home_area_pull(memory),
            ConsiderationInput::AttackFinished => flag(memory.attack_finished),
            ConsiderationInput::MotionReachFraction(motion) => self.reach_fraction(motion),
            ConsiderationInput::ActionReachFraction(action) => self.reach_fraction(action),
            ConsiderationInput::SecondsSinceTargeted => {
                self.target().map(|target| self.now - target.targeted_at)
            }
        }
    }

    /// How far past its home area range the character is. Once already heading
    /// home it reads full until arrival, so crossing back inside the range
    /// mid-return cannot hand the character straight back to wandering.
    fn home_area_pull(&self, memory: BrainMemory) -> Option<f32> {
        let home_area = self.item.home_area?;
        let position = self.item.transform.translation.xy();
        let overshoot_fraction = home_area.overshoot_fraction(position)?;
        if memory.motion == Some(Motion::ReturnToHome) {
            let arrived = home_area.distance_from(position) <= home_area.arrival_radius();
            Some(if arrived { 0.0 } else { 1.0 })
        } else {
            Some(overshoot_fraction)
        }
    }

    /// Target distance over the capability's reach; at most `1.0` is in reach.
    /// `f32::MAX` for a capability that is missing or has no reach.
    fn reach_fraction(&self, key: impl MotionActionKey) -> Option<f32> {
        let target = self.target()?;
        let reach = self.profiles[key]
            .and_then(|profile| profile.reach)
            .filter(|&reach| reach > 0.0);
        Some(reach.map_or(f32::MAX, |reach| target.distance / reach))
    }
}
