//! ECS glue for the pure utility brains.
//!
//! This module owns perceived state, the shared decision clock, and the
//! translation between a [`BrainDecision`] and equipment/motion components.
//! The brains themselves never see an `Entity`: [`model`] holds their data,
//! [`scoring`] holds the math, [`considerations`] holds what they can sense,
//! To add a motion behavior, touch these in order:
//! 1. [`model`]: a `MotionMode` variant only for a new steering contract. A
//!    one-shot commitment also goes in [`model::MotionMode::is_commitment`];
//!    target need and the arbiter bypass follow from it.
//! 2. [`motion`]: a behavior component with tuning as pub fields and memory
//!    as private fields.
 //! 3. [`model::DecisionContext`] sense fields plus [`considerations`] readers.
 //! 4. [`brains`]: static options over the mode.
//! 5. `MotionMode::group`, a [`motion::drive_motion`] match arm, and a
//!    steering helper.
//! 6. An `AiSet::Execute` applier for damage or consumed intents.

mod brains;
mod considerations;
mod model;
 mod motion;
 mod scoring;

pub use motion::{CastPlant, Dash, Strafe, Swoop, TurnRate, Wander};
use self::motion::{ANCHOR_ARRIVAL_RADIUS, drive_motion, update_dash_call, update_swoop_lottery};

pub use brains::{BAT, FIRE_MAGE, ICE_MAGE, VILLAGER, WARRIOR};
pub use model::*;
pub use scoring::*;
use brains::MELEE_HYSTERESIS;

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    prelude::{
        AttackState, Consumable, Consume, EquipmentRead, EquipmentSlot, EquipmentUsage, Health,
        Items, Mana, apply_slowed, tick_equippable_cooldown,
    },
    world::MapRng,
};

use super::{
    AiSet,
    anchor::Anchor,
    vision::{TargetInfo, Targeting},
};


/// Per-tick sensed state for one brain, written by the sense systems and read
/// by `decide`. Keeps the pure [`DecisionContext`] as the shared staging copy
/// so each sense owns only its fields.
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
struct Perception(DecisionContext);

/// A character's utility brain: which static definition it scores against,
/// the motion mode it is driving, and its private random stream.
///
/// `kind` never changes after spawn; decisions rewrite the motion fields at
/// 10 Hz while steering consumes randomness every frame.
#[derive(Component, Debug)]
#[require(Perception)]
pub struct Brain {
    pub kind: BrainKind,
    pub mode: MotionMode,
    pub committed_until: f32,
    pub dwell_until: f32,
    pub(crate) calm_phase: Option<MotionMode>,
    random: Option<ChaCha8Rng>,
}

impl Brain {
    /// Starts `kind` idle with an unseeded random stream.
    pub fn new(kind: BrainKind) -> Self {
        Self {
            kind,
            mode: MotionMode::Idle,
            committed_until: 0.0,
            dwell_until: 0.0,
            calm_phase: None,
            random: None,
        }
    }

    fn ensure_seeded(&mut self, entity: Entity, root_seed: u64) {
        if self.random.is_none() {
            let entity_seed = entity.to_bits().wrapping_mul(0x9E37_79B9_7F4A_7C15);
            self.random = Some(ChaCha8Rng::seed_from_u64(root_seed ^ entity_seed));
        }
    }

    fn generator(&mut self) -> &mut ChaCha8Rng {
        self.random
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

impl Default for Brain {
    fn default() -> Self {
        Self::new(BrainKind::Warrior)
    }
}

/// Selects one of the built-in static brains. Locomotion memory and per-tick
/// intents are opt-in at spawn; consumers query them as `Option`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrainKind {
    Warrior,
    FireMage,
    IceMage,
    Villager,
    /// Flying enemy: circles its target and breathes fire without a weapon.
    Bat,
}

impl BrainKind {
    /// Every shipped brain, in the order they are validated.
    pub const ALL: &[Self] = &[
        Self::Warrior,
        Self::FireMage,
        Self::IceMage,
        Self::Villager,
        Self::Bat,
    ];

    /// The static definition this kind scores against.
    pub(crate) fn definition(self) -> &'static BrainDefinition {
        match self {
            Self::Warrior => &WARRIOR,
            Self::FireMage => &FIRE_MAGE,
            Self::IceMage => &ICE_MAGE,
            Self::Villager => &VILLAGER,
            Self::Bat => &BAT,
        }
    }

    /// Distance band this kind's gates measure against.
    pub(crate) fn band(self) -> Option<DistanceBand> {
        match self {
            Self::Warrior => Some(MELEE_BAND),
            Self::FireMage | Self::IceMage | Self::Bat => Some(CAST_BAND),
            Self::Villager => None,
        }
    }
}


/// Latched mana-break state for casters. A single threshold would flicker on
/// and off while regeneration crosses it, so the break enters at `trigger_at`
/// and holds until the pool reaches `resume_at`.
#[derive(Component, Clone, Copy, Debug)]
pub struct ManaRecovery {
    pub trigger_at: f32,
    pub resume_at: f32,
    recovering: bool,
}

impl ManaRecovery {
    /// Enters a mana break at `trigger_at` and leaves it at `resume_at`.
    pub fn new(trigger_at: f32, resume_at: f32) -> Self {
        assert!(
            trigger_at.is_finite()
                && resume_at.is_finite()
                && trigger_at >= 0.0
                && trigger_at < resume_at
                && resume_at <= 1.0,
            "mana break needs 0 <= trigger_at < resume_at <= 1"
        );
        Self {
            trigger_at,
            resume_at,
            recovering: false,
        }
    }

    /// Updates the latch from the current mana fraction and reports the break.
    fn update(&mut self, mana_fraction: f32) -> bool {
        let mana_fraction = mana_fraction.clamp(0.0, 1.0);
        if self.recovering {
            if mana_fraction >= self.resume_at {
                self.recovering = false;
            }
        } else if mana_fraction <= self.trigger_at {
            self.recovering = true;
        }
        self.recovering
    }
}


/// Per-tick equipment intent, written by `decide_behavior` when present.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SlotIntent([bool; EquipmentSlot::ALL.len()]);
impl SlotIntent {
    pub fn get(&self, slot: EquipmentSlot) -> bool {
        self.0[slot as usize]
    }
    pub fn set_all(&mut self, intents: [bool; EquipmentSlot::ALL.len()]) {
        self.0 = intents;
    }
}
/// Whether the brain wants to drink a carried potion this tick.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct WantsToHeal(pub bool);
/// Whether the brain wants to loose its innate breath this tick.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct WantsToBreathe(pub bool);

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


/// Rejects gated options on a brain without a distance band: `pass_gate` would
/// otherwise treat every gate as open and silently ignore the eligibility rule.
fn validate_band_usage(kind: BrainKind) -> Result<(), String> {
    let definition = kind.definition();
    let gated = definition
        .motion_options
        .iter()
        .any(|option| !option.gates.is_empty())
        || definition
            .slot_options
            .iter()
            .any(|options| options.iter().any(|option| !option.gates.is_empty()))
        || definition
            .heal_options
            .iter()
            .any(|option| !option.gates.is_empty())
        || definition
            .breath_options
            .iter()
            .any(|option| !option.gates.is_empty());
    if gated && kind.band().is_none() {
        return Err("gated options require a distance band".to_string());
    }
    Ok(())
}

/// Debug-only guard against a spawned steering band drifting away from the
/// band its brain's gates measure against.
#[cfg(debug_assertions)]
fn validate_spawned_bands(characters: Query<(&Brain, &Strafe), Added<Brain>>) {
    for (brain, strafe) in &characters {
        if let Some(band) = brain.kind.band() {
            debug_assert_eq!(
                strafe.band, band,
                "{:?} spawned with a Strafe band that does not match its gates",
                brain.kind
            );
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    for &kind in BrainKind::ALL {
        if let Err(error) = validate_brain(kind.definition()) {
            panic!("{kind:?} brain is invalid: {error}");
        }
        if let Err(error) = validate_band_usage(kind) {
            panic!("{kind:?} brain is invalid: {error}");
        }
    }

    app.init_resource::<DecisionClock>()
        .init_resource::<AiSeed>()
        .add_systems(Startup, seed_ai_seed)
        .add_systems(Update, tick_decision_clock.in_set(AiSet::Decision))
        .add_systems(
            Update,
            (
                sense_target,
                sense_vitals,
                sense_equipment_and_anchor,
                sense_calm_and_attack,
            )
                .chain()
                .run_if(decision_tick_ready)
                .after(tick_decision_clock)
                .after(crate::combat::mana::regenerate_mana)
                .after(tick_equippable_cooldown)
                .in_set(AiSet::Decision),
        )
        .add_systems(
            Update,
            update_calls
                .run_if(decision_tick_ready)
                .after(sense_calm_and_attack)
                .in_set(AiSet::Decision),
        )
        .add_systems(
            Update,
            decide_behavior
                .run_if(decision_tick_ready)
                .after(update_calls)
                .in_set(AiSet::Decision),
        )
        .add_systems(
            Update,
            (apply_slot_requests, apply_heal_requests).in_set(AiSet::Execute),
        )
        .add_systems(
            Update,
            drive_motion.in_set(AiSet::Motion).after(apply_slowed),
        );

    #[cfg(debug_assertions)]
    app.add_systems(Update, validate_spawned_bands);
}

fn seed_ai_seed(map_rng: Option<Res<MapRng>>, mut ai_seed: ResMut<AiSeed>) {
    if let Some(map_rng) = map_rng {
        ai_seed.0 = map_rng.root_seed();
    } else {
        warn!("MapRng is missing; seeding AI randomness with 0");
    }
}

fn target_view(has_target: bool, info: Option<&TargetInfo>) -> Option<TargetView> {
    if !has_target {
        return None;
    }
    info.and_then(|info| {
        info.distance.is_finite().then(|| TargetView {
            distance: info.distance,
            line_of_sight: info.line_of_sight,
            bearing: info.direction,
            velocity: info.target_velocity,
            tangential_velocity: info.target_velocity.dot(info.direction.perp()),
        })
    })
}

fn slot_readiness(equipment: &EquipmentRead, entity: Entity) -> [bool; EquipmentSlot::ALL.len()] {
    let mut slot_ready = [false; EquipmentSlot::ALL.len()];
    for slot in EquipmentSlot::ALL {
        slot_ready[slot as usize] = equipment.can_use(entity, slot).is_ok();
    }
    slot_ready
}

fn vitals_fractions(
    health: Option<&Health>,
    mana: Option<&Mana>,
) -> (f32, f32) {
    let mana_fraction = mana.and_then(Mana::fraction).unwrap_or(1.0);
    let health_fraction = health.and_then(Health::fraction).unwrap_or(1.0);
    (health_fraction, mana_fraction)
}

/// Takes the one-tick attack latch, clearing it until the next attack finishes.
fn take_attack_finished(state: Option<&mut AttackState>) -> bool {
    let finished = state.as_ref().is_some_and(|state| state.finished_since_decision);
    if let Some(state) = state {
        state.finished_since_decision = false;
    }
    finished
}

fn current_calm_phase(brain: &Brain, now: f32) -> MotionMode {
    let phase = brain.calm_phase.unwrap_or(MotionMode::Idle);
    if brain.mode.is_calm() && brain.committed_until > 0.0 && now >= brain.committed_until {
        brain.mode.next_calm()
    } else {
        phase
    }
}

fn update_commit_calls(
    dash: Option<&mut Dash>,
    swoop: Option<&mut Swoop>,
    context: &mut DecisionContext,
    mode: MotionMode,
    now: f32,
    brain: &mut Brain,
) {
    // Committed calls arm off their scored triggers here; the votes below
    // only ever read the latched flags.
    if let Some(dash) = dash {
        update_dash_call(dash, context, brain);
        dash.report_called(context);
    }
    if let Some(swoop) = swoop {
        update_swoop_lottery(swoop, context, mode, now, brain);
        context.swoop_called = swoop.called;
    }
}

fn resolve_current_mode(mode: MotionMode, dash: Option<&Dash>) -> MotionMode {
    // A spent call hands the incumbent back: leaving a dead dash as the
    // incumbent would strand the warrior in the melee deadband, where
    // neither melee option is allowed to enter.
    match dash {
        Some(dash) if mode == dash.policy.mode() && !dash.is_called() => {
            dash.policy.fallback_mode().unwrap_or(mode)
        }
        _ => mode,
    }
}

fn settle_mode(
    brain: &mut Brain,
    accepted: MotionMode,
    definition: &BrainDefinition,
    now: f32,
) {
    let changed = accepted != brain.mode;
    if changed {
        brain.mode = accepted;
        brain.dwell_until = now + MIN_DWELL_SECS;
    }
    // Arm the calm-cycle window on entry, and once on spawn when the
    // initial mode is already calm. An expired window is not re-armed
    // while the mode holds: expiry is what asks the phase to advance.
    let unarmed_calm = accepted.is_calm() && brain.committed_until <= 0.0;
    if changed || unarmed_calm {
        brain.committed_until = commit_for(definition, accepted)
            .map_or(now, |window| now + brain.range(window.min(), window.max()));
    }
    // A reactive mode hands the calm cycle back to idle, which is where the
    // cycle always starts.
    let calm_phase = Some(if accepted.is_calm() {
        accepted
    } else {
        MotionMode::Idle
    });
    if brain.calm_phase != calm_phase {
        brain.calm_phase = calm_phase;
    }
}

fn cleanup_stale_calls(
    swoop: Option<&mut Swoop>,
    dash: Option<&mut Dash>,
    accepted: MotionMode,
    now: f32,
) {
    // An ended dive or a call that never converted clears here, so the
    // next lottery starts from an idle swoop instead of a stale latch.
    // `finish` also restarts the roll timer and, for a real dive, the exit
    // recovery window.
    if accepted != MotionMode::Swoop
        && let Some(swoop) = swoop
    {
        swoop.finish(now);
    }
    // A dash that lost its mode to something else (a flee, say) is
    // dropped here so a stale call cannot fire again later. The policy
    // knows which mode executes it, so the three cases collapse into one.
    if let Some(dash) = dash
        && accepted != dash.policy.mode()
    {
        dash.finish();
    }
}

fn publish_intents(
    slot_intent: Option<&mut SlotIntent>,
    heal_intent: Option<&mut WantsToHeal>,
    breathe_intent: Option<&mut WantsToBreathe>,
    decision: &BrainDecision,
) {
    if let Some(slot_intent) = slot_intent {
        let intents = decision.slot_intents;
        if slot_intent.0 != intents {
            slot_intent.set_all(intents);
        }
    }
    if let Some(heal_intent) = heal_intent {
        if heal_intent.0 != decision.wants_heal {
            heal_intent.0 = decision.wants_heal;
        }
    }
    if let Some(breathe_intent) = breathe_intent {
        if breathe_intent.0 != decision.wants_breathe {
            breathe_intent.0 = decision.wants_breathe;
        }
    }
}

fn tick_decision_clock(time: Res<Time<Virtual>>, mut clock: ResMut<DecisionClock>) {
    clock.0.tick(time.delta());
}

fn decision_tick_ready(clock: Res<DecisionClock>) -> bool {
    clock.0.just_finished()
}

/// Writes target presence and the kind's distance band; the only sense that
/// reads `BrainKind`.
fn sense_target(
    mut targets: Query<(&Brain, Has<Targeting>, Option<&TargetInfo>, &mut Perception)>,
) {
    for (brain, has_target, info, mut perception) in &mut targets {
        perception.target = target_view(has_target, info);
        perception.strafe_band = brain.kind.band();
    }
}

/// Writes health/mana fractions; the mana-break latch advances in
/// `update_calls` so this stays side-effect free.
fn sense_vitals(
    mut vitals: Query<
        (
            Option<&Health>,
            Option<&Mana>,
            &mut Perception,
        ),
        With<Brain>,
    >,
) {
    for (health, mana, mut perception) in &mut vitals {
        let (health_fraction, mana_fraction) = vitals_fractions(health, mana);
        perception.health_fraction = health_fraction;
        perception.mana_fraction = mana_fraction;
    }
}

/// Writes equipment readiness plus anchor-relative fields.
fn sense_equipment_and_anchor(
    equipment: EquipmentRead,
    mut sensed: Query<(Entity, Option<&Anchor>, &Transform, &mut Perception), With<Brain>>,
) {
    for (entity, anchor, transform, mut perception) in &mut sensed {
        perception.slot_ready = slot_readiness(&equipment, entity);
        perception.anchor_overshoot_fraction =
            anchor.and_then(|anchor| anchor.overshoot_fraction(transform.translation.xy()));
        perception.anchor_arrived = anchor.map(|anchor| {
            anchor.distance_from(transform.translation.xy()) <= ANCHOR_ARRIVAL_RADIUS
        });
    }
}

/// Writes the calm-cycle phase and takes the one-tick attack latch.
fn sense_calm_and_attack(
    time: Res<Time<Virtual>>,
    mut sensed: Query<(&Brain, Option<&mut AttackState>, &mut Perception)>,
) {
    let now = time.elapsed_secs();
    for (brain, mut attack_state, mut perception) in &mut sensed {
        perception.calm_phase = current_calm_phase(brain, now);
        perception.attack_finished = take_attack_finished(attack_state.as_deref_mut());
    }
}

/// Single writer for every call latch: pre-arm flags, mana-break update, then
/// the dash/swoop drivers report back into the context.
fn update_calls(
    time: Res<Time<Virtual>>,
    ai_seed: Res<AiSeed>,
    mut brains: Query<(
        Entity,
        &mut Brain,
        Option<&mut Dash>,
        Option<&mut Swoop>,
        Option<&Mana>,
        Option<&mut ManaRecovery>,
        &mut Perception,
    )>,
) {
    let now = time.elapsed_secs();
    for (entity, mut brain, mut dash, mut swoop, mana, mut recovery, mut perception) in &mut brains {
        brain.ensure_seeded(entity, ai_seed.0);
        let mana_fraction = mana.and_then(Mana::fraction).unwrap_or(1.0);
        // `mana_fraction` itself is owned by `sense_vitals`; only the latch
        // advances here so the field keeps a single writer.
        perception.mana_recovering = recovery
            .as_deref_mut()
            .is_some_and(|recovery| recovery.update(mana_fraction));
        perception.swoop_called = swoop.as_deref().is_some_and(|swoop| swoop.called);
        perception.kite_called = false;
        perception.burst_called = false;
        perception.slide_called = false;
        perception.dash_called = dash.as_deref().is_some_and(|dash| dash.is_called());
        perception.dash_ready = dash.as_deref().is_some_and(|dash| dash.ready(now));
        perception.dash_trigger_range = dash.as_deref().and_then(|dash| dash.policy.trigger_range());
        perception.swoop_due = swoop.as_deref().is_some_and(|swoop| now >= swoop.next_roll_at);
        update_commit_calls(
            dash.as_deref_mut(),
            swoop.as_deref_mut(),
            &mut perception.0,
            brain.mode,
            now,
            &mut brain,
        );
    }
}

/// Scores the staged perception and writes the outcome back: mode, dwell and
/// commit timers, intent components, stale-latch cleanup. Requires `Transform`
/// so membership matches the old single-query behavior.
fn decide_behavior(
    time: Res<Time<Virtual>>,
    ai_seed: Res<AiSeed>,
    mut brains: Query<(
        Entity,
        &mut Brain,
        &Perception,
        &Transform,
        Option<&mut Swoop>,
        Option<&mut Dash>,
        Option<&mut SlotIntent>,
        Option<&mut WantsToHeal>,
        Option<&mut WantsToBreathe>,
    )>,
) {
    let now = time.elapsed_secs();
    for (entity, mut brain, perception, _, mut swoop, mut dash, mut slot_intent, mut heal_intent, mut breathe_intent)
        in &mut brains
    {
        brain.ensure_seeded(entity, ai_seed.0);
        let definition = brain.kind.definition();
        let current_mode = resolve_current_mode(brain.mode, dash.as_deref());
        // Shape 3: the utility proposal is memoryless; the arbiter answers
        // "is now a good time to actually switch?" Same-group jitter waits
        // out the dwell or loses on the challenger margin, while group
        // changes and an ineligible incumbent go through at once.
        let (decision, accepted) = decide_with_arbiter(
            definition,
            &perception.0,
            current_mode,
            brain.dwell_until - now,
        );
        settle_mode(&mut brain, accepted, definition, now);
        cleanup_stale_calls(swoop.as_deref_mut(), dash.as_deref_mut(), accepted, now);
        publish_intents(
            slot_intent.as_deref_mut(),
            heal_intent.as_deref_mut(),
            breathe_intent.as_deref_mut(),
            &decision,
        );
    }
}

/// Re-gates stale intent on the live target state; skips the dead. Decisions
/// price range at 10 Hz but this runs every frame, so a target that drifted
/// out of reach between ticks must not keep a stale `try_use` alive.
fn apply_slot_requests(
    mut equipment: EquipmentUsage,
    requests: Query<(
        Entity,
        &Brain,
        &SlotIntent,
        Has<Targeting>,
        Option<&TargetInfo>,
        Option<&Health>,
    )>,
) {
    for (entity, brain, slot_intent, has_target, target_info, health) in &requests {
        if health.is_some_and(|health| health.hp <= 0.0) {
            continue;
        }
        // Live range check mirrors the slot considerations: warriors swing at
        // melee reach (plus the hold gate's hysteresis), casters fire inside
        // the cast band. Anything else keeps its decision-tick intent.
        let in_range = match brain.kind {
            BrainKind::Warrior => target_info.is_some_and(|info| {
                info.distance.is_finite()
                    && info.distance <= MELEE_RANGE + MELEE_HYSTERESIS
            }),
            BrainKind::FireMage | BrainKind::IceMage => target_info.is_some_and(|info| {
                info.distance.is_finite() && info.distance <= CAST_MAX_RANGE
            }),
            BrainKind::Villager | BrainKind::Bat => true,
        };
        for slot in EquipmentSlot::ALL {
            let sighted = target_info.is_some_and(|info| info.line_of_sight);
            let requested = has_target && sighted && in_range && slot_intent.get(slot);
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
/// Drinks a carried potion straight from the inventory; skips the dead.
fn apply_heal_requests(
    mut commands: Commands,
    requests: Query<(Entity, &WantsToHeal, Option<&Items>, Option<&Health>)>,
    consumables: Query<(), With<Consumable>>,
) {
    for (entity, heal_intent, items, health) in &requests {
        if !heal_intent.0 || health.is_some_and(|health| health.hp <= 0.0) {
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


#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn every_brain_kind_is_valid() {
        for &kind in BrainKind::ALL {
            validate_brain(kind.definition())
                .unwrap_or_else(|error| panic!("{kind:?} brain is invalid: {error}"));
            validate_band_usage(kind)
                .unwrap_or_else(|error| panic!("{kind:?} brain is invalid: {error}"));
        }
    }

    /// Spawns a circling bat with a guaranteed lottery and a live target in
    /// the cast band, then advances the clock past one decision tick.
    fn swooping_bat(chance: f32, interval: f32) -> (World, Entity) {
        let mut world = World::new();
        let mut time = Time::<Virtual>::default();
        time.advance_by(std::time::Duration::from_millis(110));
        world.insert_resource(time);
        world.insert_resource(DecisionClock::default());
        world.insert_resource(AiSeed::default());
        let target = world.spawn_empty().id();
        let bat = world
            .spawn((
                Brain {
                    kind: BrainKind::Bat,
                    mode: MotionMode::Strafe,
                    ..Default::default()
                },
                 Targeting(target),
                 TargetInfo {
                     distance: CAST_BAND.max() - 50.0,
                     direction: Vec2::X,
                     target_velocity: Vec2::ZERO,
                     line_of_sight: true,
                     in_vision_cone: true,
                 },
                 Transform::default(),
                 Strafe::new(CAST_BAND),
                 Swoop::new(chance, interval, 6.0),
            ))
            .id();
        (world, bat)
    }
    fn bat_mode(world: &mut World, bat: Entity) -> MotionMode {
        world.get::<Brain>(bat).expect("brained bat").mode
    }
    /// The full loop holds: a called tick proposes the dive and the arbiter
    /// accepts it out of the circle the same tick.
    #[test]
    fn swoop_lottery_drives_a_dive() {
        let (mut world, bat) = swooping_bat(1.0, 0.0);
        world.run_system_once(sense_target).expect("target sensed");
        world.run_system_once(sense_vitals).expect("vitals sensed");
        world.run_system_once(sense_equipment_and_anchor).expect("equipment sensed");
        world.run_system_once(sense_calm_and_attack).expect("calm sensed");
        world.run_system_once(update_calls).expect("calls updated");
        world.run_system_once(decide_behavior).expect("decisions run");
        assert_eq!(bat_mode(&mut world, bat), MotionMode::Swoop);
    }
}
