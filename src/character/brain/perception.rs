//! What a brain knows: vision tracks a target, and observing turns world state
//! into the [`DecisionContext`] the considerations read.

use std::f32::consts::FRAC_PI_4;

use avian2d::prelude::{RayCaster, RayHits};
use bevy::{ecs::query::QueryData, ecs::system::SystemParam, prelude::*};

use super::action::{Ability, Action, find_carried_potion};
use super::capability::{Capabilities, MotionActionTable, capability_keys};
use super::motion::{Anchor, Motion};
use crate::{character::AiSet, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_damage_aggro)
        .add_observer(on_targeting_added_record_time)
        .add_systems(
            Update,
            (
                (update_target_info, is_target_in_sight)
                    .chain()
                    .in_set(AiSet::Perception),
                (should_target_watched, should_stop_targeting).in_set(AiSet::Targeting),
                update_aim_position.in_set(AiSet::Aim),
            ),
        );
}

/// Target data visible to a brain this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TargetView {
    pub distance: f32,
    pub line_of_sight: bool,
    /// Unit direction from the viewer to the target.
    pub bearing: Vec2,
    pub velocity: Vec2,
    pub seconds_since_targeted: f32,
}

/// Everything a pure brain may look at during one decision.
#[derive(Clone, Copy, Debug, Default)]
pub struct DecisionContext {
    pub me: SelfView,
    pub target: Option<TargetView>,
    pub reach_fraction: MotionActionTable<Option<f32>>,
    pub memory: MemoryView,
}

/// The deciding character's own state this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelfView {
    pub health_fraction: f32,
    /// `1.0` for a non-caster.
    pub mana_fraction: f32,
    pub stunned: bool,
    pub anchor: Option<AnchorView>,
}

impl Default for SelfView {
    fn default() -> Self {
        Self {
            health_fraction: 1.0,
            mana_fraction: 1.0,
            stunned: false,
            anchor: None,
        }
    }
}

/// Where the character stands relative to the anchor it is tethered to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchorView {
    pub overshoot_fraction: f32,
    pub arrived: bool,
}

/// The brain's own earlier choices, as scoring sees them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MemoryView {
    pub motion: Option<Motion>,
    /// An attack finished since the last decision.
    pub attack_finished: bool,
}

/// The raw fact a consideration reads; see [`read_input`].
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
    /// How far past its anchor range the character is.
    AnchorPull,
    AttackFinished,
    /// Target distance over this motion's reach; at most `1.0` is in reach.
    MotionReachFraction(Motion),
    /// Target distance over this action's reach; at most `1.0` is in reach.
    ActionReachFraction(Action),
    SecondsSinceTargeted,
}

/// Reads `input` from the context; `None` when the input is absent.
pub(super) fn read_input(
    input: ConsiderationInput,
    decision_context: &DecisionContext,
) -> Option<f32> {
    let boolean_input = |set: bool| Some(if set { 1.0 } else { 0.0 });
    match input {
        ConsiderationInput::HasTarget => boolean_input(decision_context.target.is_some()),
        ConsiderationInput::LineOfSight => decision_context
            .target
            .and_then(|target| boolean_input(target.line_of_sight)),
        ConsiderationInput::TargetDistance => decision_context.target.map(|target| target.distance),
        ConsiderationInput::MissingHealth => Some(1.0 - decision_context.me.health_fraction),
        ConsiderationInput::ManaFraction => Some(decision_context.me.mana_fraction),
        ConsiderationInput::MotionRunning(motion) => boolean_input(
            decision_context
                .memory
                .motion
                .is_some_and(|running| running.index() == motion.index()),
        ),
        ConsiderationInput::Circling => boolean_input(matches!(
            decision_context.memory.motion,
            Some(Motion::Orbit(_) | Motion::KeepDistance(_))
        )),
        // Once already heading home it reads full until arrival, so crossing
        // back inside the range mid-return cannot hand the character straight
        // back to calm.
        ConsiderationInput::AnchorPull => {
            let anchor = decision_context.me.anchor?;
            if decision_context.memory.motion == Some(Motion::ReturnToAnchor) {
                boolean_input(!anchor.arrived)
            } else {
                Some(anchor.overshoot_fraction)
            }
        }
        ConsiderationInput::AttackFinished => {
            boolean_input(decision_context.memory.attack_finished)
        }
        ConsiderationInput::MotionReachFraction(motion) => decision_context.reach_fraction[motion],
        ConsiderationInput::ActionReachFraction(action) => decision_context.reach_fraction[action],
        ConsiderationInput::SecondsSinceTargeted => decision_context
            .target
            .map(|target| target.seconds_since_targeted),
    }
}

/// How wide an entity's vision cone is.
#[derive(Component)]
#[component(immutable)]
#[require(Vision, TargetInfo)]
pub(in crate::character) struct VisionCapabilities {
    /// Half-angle of the field of view in radians; `PI / 4.0` is a 90° cone.
    pub vision_cone_radius: f32,
}

impl Default for VisionCapabilities {
    fn default() -> Self {
        Self {
            vision_cone_radius: FRAC_PI_4,
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
#[derive(Component)]
#[relationship(relationship_target = WatchedBy)]
pub(in crate::character) struct Watching(pub Entity);

/// Inverse of [`Watching`].
#[derive(Component)]
#[relationship_target(relationship = Watching)]
pub(in crate::character) struct WatchedBy(Vec<Entity>);

/// Keeps `Targeting` without sight until its scheduled removal.
#[derive(Component)]
struct TargetLock;

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
    character_query
        .par_iter_mut()
        .for_each(|(mut vision, target_info, has_target, facing_dir)| {
            vision.aim_direction = if has_target {
                target_info.direction
            } else {
                facing_dir.to_vec2()
            };
        });
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
    target_query: Query<(&Transform, Option<&SimpleMotion>)>,
) {
    npc_query.par_iter_mut().for_each(
        |(mut target_info, mut ray_caster, transform, watching, targeting)| {
            let target_entity = targeting.map_or(watching.0, |t| t.0);

            if let Ok((target_transform, target_motion)) = target_query.get(target_entity) {
                let target_direction = (target_transform.translation.xy()
                    - transform.translation.xy())
                .normalize_or_zero();

                let target_distance = target_transform
                    .translation
                    .xy()
                    .distance(transform.translation.xy());

                target_info.direction = target_direction;
                target_info.distance = target_distance;
                target_info.target_velocity =
                    target_motion.map_or(Vec2::ZERO, SimpleMotion::get_velocity);

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
        &VisionCapabilities,
        &Watching,
        Option<&Targeting>,
    )>,
    target_query: Query<Option<&Children>, Or<(With<WatchedBy>, With<TargetedBy>)>>,
) {
    npc_query.par_iter_mut().for_each(
        |(mut target_info, ray_hits, vision, vision_capabilities, watching, targeting)| {
            target_info.line_of_sight = false;

            let vision_cone_dot = vision_capabilities.vision_cone_radius.cos();
            target_info.in_vision_cone =
                target_info.direction.dot(vision.aim_direction) > vision_cone_dot;

            let target_entity = targeting.map_or(watching.0, |t| t.0);

            for hit in ray_hits {
                if hit.entity == target_entity {
                    target_info.line_of_sight = true;
                    break;
                }

                if let Ok(Some(children)) = target_query.get(target_entity)
                    && children.contains(&hit.entity)
                {
                    target_info.line_of_sight = true;
                    break;
                }
            }
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
    added: On<Add, Targeting>,
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
    npc_query
        .iter()
        .for_each(|(target_info, watching, entity)| {
            if target_info.line_of_sight && target_info.in_vision_cone {
                commands.entity(entity).insert(Targeting(watching.0));
            }
        });
}

/// Stops targeting if the entity loses sight of the target or the target lock has expired.
fn should_stop_targeting(
    mut commands: Commands,
    npc_query: Query<(&TargetInfo, Has<TargetLock>, Entity), With<Targeting>>,
) {
    npc_query
        .iter()
        .for_each(|(target_info, has_lock, entity)| {
            if !target_info.line_of_sight && !has_lock {
                commands.entity(entity).remove::<Targeting>();
            }
        });
}

/// Builds the pure view of a target from the vision system's observation.
pub(super) fn build_target_view(
    has_target: bool,
    info: Option<&TargetInfo>,
    now: f32,
) -> Option<TargetView> {
    if !has_target {
        return None;
    }
    info.and_then(|info| {
        info.distance.is_finite().then_some(TargetView {
            distance: info.distance,
            line_of_sight: info.line_of_sight,
            bearing: info.direction,
            velocity: info.target_velocity,
            seconds_since_targeted: now - info.targeted_at,
        })
    })
}

/// Target distance over each owned capability's reach; `f32::MAX` for a
/// capability that is missing or has no reach.
fn build_reach_fractions(
    target: Option<TargetView>,
    capabilities: &Capabilities,
) -> MotionActionTable<Option<f32>> {
    let Some(target) = target else {
        return MotionActionTable::default();
    };
    let mut reach_fraction = MotionActionTable::splat(Some(f32::MAX));
    for key in capability_keys() {
        if let Some(reach) = capabilities[key]
            .and_then(|tuning| tuning.reach)
            .filter(|&reach| reach > 0.0)
        {
            reach_fraction[key] = Some(target.distance / reach);
        }
    }
    reach_fraction
}

/// The queries behind each capability's world-allows-use check, shared by
/// `observe_world_facts` and the appliers.
#[derive(SystemParam)]
pub(super) struct WorldUseChecks<'w, 's> {
    pub equipment: EquipmentRead<'w, 's>,
    pub consumables: Query<'w, 's, (), With<Consumable>>,
    pub items_in_use: Query<'w, 's, (), With<EquipmentInUse>>,
}

/// Whether the world lets each motion or action run now: usable equipment (or
/// a held use already running), and a carried potion for `Heal`.
fn build_world_allows_use(
    entity: Entity,
    items: Option<&Items>,
    use_checks: &WorldUseChecks,
) -> MotionActionTable<bool> {
    let mut world_allows_use = MotionActionTable::splat(true);
    for slot in EquipmentSlot::ALL {
        let held_item_in_use = !slot.is_instant_use()
            && use_checks
                .equipment
                .item_in_slot(entity, slot)
                .is_some_and(|item| use_checks.items_in_use.contains(item));
        world_allows_use[Action::Use(slot)] =
            held_item_in_use || use_checks.equipment.can_use(entity, slot).is_ok();
    }
    world_allows_use[Ability::Heal] = find_carried_potion(items, &use_checks.consumables).is_some();
    world_allows_use
}

/// The world-fact components one decision reads.
#[derive(QueryData)]
pub(super) struct ObservedComponents {
    entity: Entity,
    transform: &'static Transform,
    has_target: Has<Targeting>,
    target_info: Option<&'static TargetInfo>,
    health: Option<&'static Health>,
    mana: Option<&'static Mana>,
    anchor: Option<&'static Anchor>,
    movement: Option<&'static SimpleMotion>,
    items: Option<&'static Items>,
}

/// Reads one decision's world facts: the context, with memory left for the
/// brain to add, and whether the world lets each motion or action run now.
pub(super) fn observe_world_facts(
    components: &ObservedComponentsItem,
    capabilities: &Capabilities,
    use_checks: &WorldUseChecks,
    now: f32,
) -> (DecisionContext, MotionActionTable<bool>) {
    let position = components.transform.translation.xy();
    let target = build_target_view(components.has_target, components.target_info, now);
    let anchor = components.anchor.and_then(|anchor| {
        Some(AnchorView {
            overshoot_fraction: anchor.overshoot_fraction(position)?,
            arrived: anchor.distance_from(position) <= anchor.arrival_radius(),
        })
    });
    let context = DecisionContext {
        me: SelfView {
            health_fraction: components.health.and_then(Health::fraction).unwrap_or(1.0),
            mana_fraction: components.mana.and_then(Mana::fraction).unwrap_or(1.0),
            stunned: components.movement.is_some_and(SimpleMotion::is_stunned),
            anchor,
        },
        target,
        reach_fraction: build_reach_fractions(target, capabilities),
        memory: MemoryView::default(),
    };
    let world_allows_use = build_world_allows_use(components.entity, components.items, use_checks);
    (context, world_allows_use)
}
