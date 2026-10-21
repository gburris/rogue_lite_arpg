//! ECS glue for the pure utility brains.
//!
//! This module owns perceived state, the shared decision clock, and the
//! translation between a [`BrainDecision`] and equipment/motion components.
//! The brains themselves never see an `Entity`: [`model`] holds their data,
//! [`scoring`] holds the math, [`considerations`] holds what they can sense,
//! and [`brains`] holds the shipped definitions.

mod brains;
mod considerations;
mod model;
mod scoring;

pub use brains::{RANGED_MAGE, VILLAGER, WARRIOR};
pub use model::*;
pub use scoring::*;

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    prelude::{
        Consume, Consumable, EquipmentRead, EquipmentSlot, EquipmentUsage, Health, Items,
        TILE_SIZE, apply_slowed, tick_equippable_cooldown,
    },
    world::MapRng,
};

use super::{
    AiSet,
    anchor::Anchor,
    state::SimpleMotion,
    vision::{TargetInfo, Targeting},
};

const ANCHOR_ARRIVAL_RADIUS: f32 = TILE_SIZE as f32 / 2.0;
/// Wander speed as a fraction of max speed. Full-speed wandering covers more
/// ground per step than small anchor ranges allow (an NPC walks 100-250 px
/// per step inside an 80 px range), which turns every stroll into a march
/// home. Ambling keeps most steps inside the range.
const WANDER_SPEED_SCALE: f32 = 0.4;

/// Selects one of the built-in static brains.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
#[require(MotionState, WanderPlan, SlotIntent, HealIntent, BrainRandom)]
pub enum BrainKind {
    Warrior,
    RangedMage,
    Villager,
}

impl BrainKind {
    /// The static definition this kind scores against.
    pub(crate) fn definition(self) -> &'static Brain {
        match self {
            Self::Warrior => &WARRIOR,
            Self::RangedMage => &RANGED_MAGE,
            Self::Villager => &VILLAGER,
        }
    }

    /// How long one wander direction is held before choosing another.
    ///
    /// These ranges match the old wander timers: enemies changed direction
    /// every 1–2 seconds, villagers every 1–2.5 seconds.
    pub(crate) fn wander_step_range(self) -> (f32, f32) {
        match self {
            Self::Villager => (1.0, 2.5),
            Self::Warrior | Self::RangedMage => (1.0, 2.0),
        }
    }

    /// First half of this kind's calm idle/wander cycle.
    fn initial_calm_mode(self) -> MotionMode {
        match self {
            Self::Villager => MotionMode::Idle,
            Self::Warrior | Self::RangedMage => MotionMode::Wander,
        }
    }

    /// The other half of the calm cycle, or the first half after a reactive mode.
    fn next_calm_mode(self, current: MotionMode) -> MotionMode {
        match current {
            MotionMode::Idle => MotionMode::Wander,
            MotionMode::Wander => MotionMode::Idle,
            _ => self.initial_calm_mode(),
        }
    }
}

/// The motion mode currently being driven, plus its commitment window.
#[derive(Component, Debug, Default)]
pub struct MotionState {
    pub mode: MotionMode,
    pub committed_until: f32,
    calm_phase: Option<MotionMode>,
}

/// Locomotion scratch: wander direction/expiry, reused for separation nudges. The scorer never sees these.
#[derive(Component, Default)]
pub struct WanderPlan {
    direction: Vec2,
    next_direction_at: f32,
}

/// Per-tick equipment intent produced by `run_decisions` and consumed by
/// `apply_slot_requests`.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SlotIntent([bool; EquipmentSlot::ALL.len()]);
impl SlotIntent {
    /// Whether the brain requested `slot` on its last decision tick.
    pub fn get(&self, slot: EquipmentSlot) -> bool {
        self.0[slot as usize]
    }

    /// Replaces the whole intent with a fresh decision tick's output.
    pub fn set_all(&mut self, intents: [bool; EquipmentSlot::ALL.len()]) {
        self.0 = intents;
    }
}
/// Whether the brain wants to drink a carried potion this tick, consumed by
/// `apply_heal_requests` straight from the inventory.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct HealIntent(pub bool);

/// Shared ~10 Hz decision clock.
#[derive(Resource)]
pub struct DecisionClock(pub Timer);

impl Default for DecisionClock {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

/// Root seed used to seed each brain's independent random stream.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct AiSeed(pub u64);

/// Per-entity randomness, seeded deterministically from the root seed and the
/// entity so one brain's rolls cannot shift another's.
#[derive(Component, Default)]
pub struct BrainRandom(Option<ChaCha8Rng>);

impl BrainRandom {
    /// Creates a standalone generator, mainly for tests.
    #[cfg(test)]
    pub fn seeded(seed: u64) -> Self {
        Self(Some(ChaCha8Rng::seed_from_u64(seed)))
    }

    /// Seeds this entity's stream on its first use.
    fn ensure_seeded(&mut self, entity: Entity, root_seed: u64) {
        if self.0.is_none() {
            let entity_seed = entity.to_bits().wrapping_mul(0x9E37_79B9_7F4A_7C15);
            self.0 = Some(ChaCha8Rng::seed_from_u64(root_seed ^ entity_seed));
        }
    }

    fn generator(&mut self) -> &mut ChaCha8Rng {
        self.0
            .as_mut()
            .expect("brain randomness is seeded before it is read")
    }

    /// Uniform value in `min..max`, or `min` for an empty/invalid range.
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        if max > min {
            self.generator().random_range(min..max)
        } else {
            min
        }
    }

    /// A uniformly random unit direction.
    pub fn direction(&mut self) -> Vec2 {
        let angle = self.generator().random_range(0.0..std::f32::consts::TAU);
        Vec2::new(angle.cos(), angle.sin())
    }
}

pub(super) fn plugin(app: &mut App) {
    for kind in [
        BrainKind::Warrior,
        BrainKind::RangedMage,
        BrainKind::Villager,
    ] {
        if let Err(error) = validate_brain(kind.definition()) {
            panic!("{kind:?} brain is invalid: {error}");
        }
    }

    app.init_resource::<DecisionClock>()
        .init_resource::<AiSeed>()
        .add_systems(Startup, seed_ai_seed)
        .add_systems(
            Update,
            (
                run_decisions
                    .after(crate::combat::mana::regenerate_mana)
                    .after(tick_equippable_cooldown),
                apply_slot_requests,
                apply_heal_requests,
            )
                .chain()
                .in_set(AiSet::Decision),
        )
        .add_systems(
            Update,
            drive_motion.in_set(AiSet::Motion).after(apply_slowed),
        );
}

/// Copies the map's root seed into the AI seed once all plugins are built.
fn seed_ai_seed(map_rng: Option<Res<MapRng>>, mut ai_seed: ResMut<AiSeed>) {
    if let Some(map_rng) = map_rng {
        ai_seed.0 = map_rng.root_seed();
    } else {
        warn!("MapRng is missing; seeding AI randomness with 0");
    }
}

/// Gathers a neutral-defaulted context and runs the pure scorer.
fn run_decisions(
    time: Res<Time<Virtual>>,
    mut clock: ResMut<DecisionClock>,
    ai_seed: Res<AiSeed>,
    equipment: EquipmentRead,
    mut brains: Query<(
        Entity,
        &BrainKind,
        &mut MotionState,
        Has<Targeting>,
        Option<&TargetInfo>,
        &Transform,
        Option<&Health>,
        Option<&Anchor>,
        &mut SlotIntent,
        &mut HealIntent,
        &mut BrainRandom,
    )>,
) {
    if !clock.0.tick(time.delta()).just_finished() {
        return;
    }

    let now = time.elapsed_secs();

    for (
        entity,
        kind,
        mut motion,
        has_target,
        target_info,
        transform,
        health,
        anchor,
        mut slot_intent,
        mut heal_intent,
        mut random,
    ) in &mut brains
    {
        random.ensure_seeded(entity, ai_seed.0);

        let target = if has_target {
            target_info.and_then(|info| {
                info.distance.is_finite().then(|| TargetView {
                    distance: info.distance,
                    line_of_sight: info.line_of_sight,
                })
            })
        } else {
            None
        };

        let mut slot_ready = [false; EquipmentSlot::ALL.len()];
        for slot in EquipmentSlot::ALL {
            slot_ready[slot as usize] = equipment.can_use(entity, slot).is_ok();
        }

        let mut calm_phase = motion
            .calm_phase
            .unwrap_or_else(|| kind.initial_calm_mode());
        if motion.mode.is_calm() && motion.committed_until > 0.0 && now >= motion.committed_until {
            calm_phase = kind.next_calm_mode(motion.mode);
        }

        let context = DecisionContext {
            target,
            slot_ready,
            health_fraction: health.and_then(Health::fraction).unwrap_or(1.0),
            anchor_overshoot_fraction: anchor
                .and_then(|anchor| anchor.overshoot_fraction(transform.translation.xy())),
            anchor_arrived: anchor.map(|anchor| {
                anchor.distance_from(transform.translation.xy()) <= ANCHOR_ARRIVAL_RADIUS
            }),
            calm_phase,
            within_commitment: now < motion.committed_until,
        };

        let decision = decide(kind.definition(), &context, motion.mode);

        if decision.motion != motion.mode {
            motion.mode = decision.motion;
            motion.committed_until = decision
                .commit
                .map_or(now, |window| now + random.range(window.min, window.max));
        } else if now >= motion.committed_until
            && let Some(window) = decision.commit
        {
            motion.committed_until = now + random.range(window.min, window.max);
        }

        motion.calm_phase = Some(if decision.motion.is_calm() {
            decision.motion
        } else {
            kind.initial_calm_mode()
        });

        slot_intent.set_all(decision.slot_intents);
        heal_intent.0 = decision.wants_heal;
    }
}

/// Applies slot intent through the same equipment API the player uses.
/// Instant uses are never cancelled by a decision flip; held slots are.
///
/// Intent is re-gated on the current `Targeting` state because decisions only
/// refresh every 100 ms. Without this, an intent computed while a target was
/// alive could still fire one tick after the target is lost.
fn apply_slot_requests(
    mut equipment: EquipmentUsage,
    requests: Query<(Entity, &SlotIntent, Has<Targeting>, Option<&TargetInfo>), With<BrainKind>>,
) {
    for (entity, slot_intent, has_target, target_info) in &requests {
        for slot in EquipmentSlot::ALL {
            let sighted = target_info.is_some_and(|info| info.line_of_sight);
            let requested = has_target && sighted && slot_intent.get(slot);
            let in_use = equipment.is_using(entity, slot);

            if requested && !in_use {
                if equipment.can_use(entity, slot).is_ok() {
                    let _ = equipment.try_use(entity, slot);
                }
            } else if !requested && in_use && !slot.is_instant_use() {
                equipment.stop_using(entity, slot);
            }
        }
    }
}
/// Drinks a carried potion once the brain calls for it. Potions live in the
/// inventory and are consumed directly, exactly like the player's
/// click-to-drink path — never equipped.
fn apply_heal_requests(
    mut commands: Commands,
    requests: Query<(Entity, &HealIntent, Option<&Items>), With<BrainKind>>,
    consumables: Query<(), With<Consumable>>,
) {
    for (entity, heal_intent, items) in &requests {
        if !heal_intent.0 {
            continue;
        }
        let potion = items
            .iter()
            .flat_map(|items| items.iter())
            .find(|item| consumables.contains(*item));
        if let Some(potion) = potion {
            commands.trigger(Consume {
                entity,
                item_entity: potion,
            });
        }
    }
}

/// Picks a fresh wander direction. Deep inside the anchor range any direction
/// will do; near or past the edge the choices bend home with some slack, so
/// the walk reads as meandering instead of an about-face march to the origin.
fn pick_wander_direction(
    random: &mut BrainRandom,
    anchor: Option<&Anchor>,
    position: Vec2,
) -> Vec2 {
    let candidate = random.direction();
    let Some(anchor) = anchor else {
        return candidate;
    };
    if anchor
        .overshoot_fraction(position)
        .is_none_or(|fraction| fraction < -0.25)
    {
        return candidate;
    }
    let home = anchor.origin() - position;
    if home == Vec2::ZERO {
        return candidate;
    }
    home.normalize_or_zero()
        .rotate(Vec2::from_angle(random.range(-0.6, 0.6)))
}

/// Drives `SimpleMotion` from the selected mode. Motion never waits on
/// equipment, and equipment use never waits on motion.
fn drive_motion(
    time: Res<Time<Virtual>>,
    ai_seed: Res<AiSeed>,
    mut characters: Query<
        (
            Entity,
            &BrainKind,
            &MotionState,
            &mut SimpleMotion,
            &Transform,
            Option<&TargetInfo>,
            Has<Targeting>,
            Option<&Anchor>,
            &mut WanderPlan,
            &mut BrainRandom,
        ),
        With<BrainKind>,
    >,
) {
    let now = time.elapsed_secs();

    for (
        entity,
        kind,
        motion_state,
        mut motion,
        transform,
        target_info,
        has_target,
        anchor,
        mut wander,
        mut random,
    ) in &mut characters
    {
        random.ensure_seeded(entity, ai_seed.0);

        if motion_state.mode.needs_target() && (!has_target || target_info.is_none()) {
            motion.stop_moving();
            continue;
        }

        match motion_state.mode {
            MotionMode::Idle => motion.stop_moving(),
            MotionMode::Chase => drive_direction(&mut motion, chase_direction(target_info), 1.0),
            // Still in the fight: run from the target, not toward home.
            // Going home happens only once the target is lost.
            MotionMode::Flee => drive_direction(&mut motion, flee_direction(target_info), 1.0),
            MotionMode::ReturnToAnchor => drive_direction(
                &mut motion,
                anchor_direction(transform.translation.xy(), anchor),
                1.0,
            ),
            MotionMode::KeepDistance { min, max } => drive_direction(
                &mut motion,
                keep_distance_direction(target_info, min, max, &mut wander, now, &mut random),
                1.0,
            ),
            MotionMode::Strafe => {
                drive_direction(&mut motion, strafe_direction(entity, target_info), 1.0)
            }
            MotionMode::Wander => drive_direction(
                &mut motion,
                Some(wander_direction(
                    &mut wander,
                    kind.wander_step_range(),
                    anchor,
                    transform.translation.xy(),
                    now,
                    &mut random,
                )),
                WANDER_SPEED_SCALE,
            ),
        }
    }
}

/// Applies a steering decision at the given speed scale; `None` stops.
fn drive_direction(motion: &mut SimpleMotion, direction: Option<Vec2>, speed_scale: f32) {
    match direction {
        Some(direction) => motion.start_moving_scaled(direction, speed_scale),
        None => motion.stop_moving(),
    }
}
/// Unit vector toward the target, or `None` when the bearing is unknown.
fn chase_direction(target_info: Option<&TargetInfo>) -> Option<Vec2> {
    let direction = target_info.map_or(Vec2::ZERO, |info| info.direction);
    (direction.length_squared() > f32::EPSILON).then_some(direction)
}
/// Unit vector away from the target, or `None` when the bearing is unknown.
fn flee_direction(target_info: Option<&TargetInfo>) -> Option<Vec2> {
    chase_direction(target_info).map(|direction| -direction)
}
/// Unit vector toward the anchor origin, or `None` once arrived (or anchorless).
fn anchor_direction(position: Vec2, anchor: Option<&Anchor>) -> Option<Vec2> {
    let anchor = anchor?;
    let to_anchor = anchor.origin() - position;
    (to_anchor.length() > ANCHOR_ARRIVAL_RADIUS).then_some(to_anchor.normalize_or_zero())
}
/// Sidestep around the target, or `None` when the bearing is unknown.
/// Handedness is fixed per entity so strafers don't mirror each other off
/// one shared random stream.
fn strafe_direction(entity: Entity, target_info: Option<&TargetInfo>) -> Option<Vec2> {
    let direction = chase_direction(target_info)?;
    let sign = if entity.to_bits() % 2 == 0 { 1.0 } else { -1.0 };
    Some(direction.perp() * sign)
}
/// Holds the `[min, max]` band around the target, or `None` when inside it.
/// Overlapping the target leaves no direction to steer by — toward and away
/// are both zero — so it holds a brief separation nudge instead of freezing
/// while contact separation finishes the job.
fn keep_distance_direction(
    target_info: Option<&TargetInfo>,
    min: f32,
    max: f32,
    wander: &mut WanderPlan,
    now: f32,
    random: &mut BrainRandom,
) -> Option<Vec2> {
    let distance = target_info.map_or(0.0, |info| info.distance);
    let direction = target_info.map_or(Vec2::ZERO, |info| info.direction);
    if direction.length_squared() <= f32::EPSILON {
        if distance < min {
            if now >= wander.next_direction_at {
                wander.direction = random.direction();
                wander.next_direction_at = now + 0.2;
            }
            return Some(wander.direction);
        }
        return None;
    }
    if distance < min {
        Some(-direction)
    } else if distance > max {
        Some(direction)
    } else {
        None
    }
}
/// Current wander direction, rolling a fresh one when the step expires.
fn wander_direction(
    wander: &mut WanderPlan,
    step_range: (f32, f32),
    anchor: Option<&Anchor>,
    position: Vec2,
    now: f32,
    random: &mut BrainRandom,
) -> Vec2 {
    if now >= wander.next_direction_at {
        let (minimum, maximum) = step_range;
        wander.direction = pick_wander_direction(random, anchor, position);
        wander.next_direction_at = now + random.range(minimum, maximum);
    }
    wander.direction
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_brain_kind_is_valid() {
        for kind in [
            BrainKind::Warrior,
            BrainKind::RangedMage,
            BrainKind::Villager,
        ] {
            validate_brain(kind.definition())
                .unwrap_or_else(|error| panic!("{kind:?} brain is invalid: {error}"));
        }
    }

    #[test]
    fn fixed_seed_repeats_identically() {
        let mut first = BrainRandom::seeded(7);
        let mut second = BrainRandom::seeded(7);

        for _ in 0..32 {
            assert_eq!(first.direction(), second.direction());
            assert_eq!(
                first.range(0.5, 1.5).to_bits(),
                second.range(0.5, 1.5).to_bits()
            );
        }
    }
    #[test]
    fn wander_direction_bends_home_near_the_edge() {
        let anchor = Anchor::new(Vec2::ZERO, 100.0);
        let mut random = BrainRandom::seeded(3);
        let inner = pick_wander_direction(&mut random, Some(&anchor), Vec2::new(10.0, 10.0));
        assert!((inner.length() - 1.0).abs() < 1e-5);

        // Past the edge every roll must point home-ish, never away.
        let position = Vec2::new(150.0, 20.0);
        let home = (Vec2::ZERO - position).normalize();
        for _ in 0..16 {
            let direction = pick_wander_direction(&mut random, Some(&anchor), position);
            assert!(direction.dot(home) > 0.0);
        }

        let free = pick_wander_direction(&mut random, None, Vec2::new(500.0, 0.0));
        assert!((free.length() - 1.0).abs() < 1e-5);
    }
}
