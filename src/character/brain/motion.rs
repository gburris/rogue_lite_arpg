//! Per-frame steering execution: translates [`Brain`](super::Brain) motion into [`SimpleMotion`].
//!
//! [`super`] decides at ~10 Hz what to do; this module drives every frame how
//! to move. Steering memories ([`Wander`], [`Strafe`], [`Swoop`], [`Dash`])
//! are opt-in capabilities that persist across mode switches, so they stay as
//! separate components rather than one enum. Committed straight dashes share
//! one execution and differ only in what calls them ([`DashPolicy`]).

use bevy::prelude::*;

use super::brains::{TRIGGER_BREAKAWAY, TRIGGER_ESCAPE, TRIGGER_SWOOP, TRIGGER_VOLLEY_HOP};
use super::model::{
    DecisionContext, DistanceBand, MotionMode, ScoredOption, TARGET_MOVING_SPEED, TargetView,
};
use super::scoring::score_option;
use super::{AiSeed, Brain};
use super::super::{anchor::Anchor, state::SimpleMotion, vision::{TargetInfo, Targeting}};
use crate::prelude::{AttackState, TILE_SIZE};
pub(crate) const ANCHOR_ARRIVAL_RADIUS: f32 = TILE_SIZE as f32 / 2.0;

/// Wander speed as a fraction of max speed; ambling keeps steps inside small anchors.
const WANDER_SPEED_SCALE: f32 = 0.4;
const STRAFE_REROLL_MIN: f32 = 1.5;
const STRAFE_REROLL_MAX: f32 = 2.5;
const STRAFE_SWITCH_CHANCE: f32 = 0.5;
const STRAFE_RADIAL_GAIN: f32 = 0.5;
const SMART_STRAFE_REROLL_MIN: f32 = 0.5;
const SMART_STRAFE_REROLL_MAX: f32 = 0.9;
const SMART_STRAFE_INPUT_DEADBAND: f32 = 20.0;
const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// Half-angle of a warrior's break cone around straight away from its target.
const KITE_SPREAD: f32 = std::f32::consts::FRAC_PI_4;

/// Half-angle of the fire mage's step-in spread around the target bearing.
const BURST_STEP_IN_SPREAD: f32 = 0.35;
/// Small outward bias on a tangent burst so a volley does not walk the mage
/// into the target when the band is already tight.
const BURST_OUTWARD_BIAS: f32 = 0.15;
/// Maximum turn rate in radians per second. Absent means snap.
#[derive(Component, Clone, Copy, Debug)]
pub struct TurnRate(pub f32);

/// How one entity wanders. Opt-in at spawn; absent means that entity never wanders.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wander {
    pub min: f32,
    pub max: f32,
    pub speed_scale: f32,
    direction: Vec2,
    next_direction_at: f32,
}
impl Wander {
    /// Holds each direction for a uniform roll in `min..=max` seconds.
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            speed_scale: WANDER_SPEED_SCALE,
            direction: Vec2::ZERO,
            next_direction_at: 0.0,
        }
    }
}
/// How one entity strafes. Opt-in at spawn; absent means never strafes.
#[derive(Component, Clone, Copy, Debug)]
pub struct Strafe {
    pub band: DistanceBand,
    pub reroll_min: f32,
    pub reroll_max: f32,
    pub switch_chance: f32,
    pub radial_gain: f32,
    pub standoff_range: f32,
    /// Whether the orbit side should follow the target's tangential movement.
    pub smart_orbit: bool,
    standoff: f32,
    orbit: Option<Orbit>,
    reroll_at: f32,
}
impl Strafe {
    /// Orbits `band` with the default rhythm and no radius standoff.
    pub fn new(band: DistanceBand) -> Self {
        Self {
            band,
            reroll_min: STRAFE_REROLL_MIN,
            reroll_max: STRAFE_REROLL_MAX,
            switch_chance: STRAFE_SWITCH_CHANCE,
            radial_gain: STRAFE_RADIAL_GAIN,
            standoff_range: 0.0,
            smart_orbit: false,
            standoff: 0.0,
            orbit: None,
            reroll_at: 0.0,
        }
    }

    /// Orbits with a short reaction window and counter-moves the target's
    /// tangential motion instead of rerolling the side at random.
    pub fn smart(band: DistanceBand) -> Self {
        Self {
            reroll_min: SMART_STRAFE_REROLL_MIN,
            reroll_max: SMART_STRAFE_REROLL_MAX,
            switch_chance: 0.0,
            smart_orbit: true,
            ..Self::new(band)
        }
    }

    /// Also rerolls a radius offset in `-standoff_range..=standoff_range`.
    pub fn with_standoff(band: DistanceBand, standoff_range: f32) -> Self {
        Self {
            standoff_range,
            ..Self::new(band)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Orbit {
    #[default]
    Clockwise,
    CounterClockwise,
}

impl Orbit {
    fn flipped(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }

    fn sign(self) -> f32 {
        match self {
            Self::Clockwise => -1.0,
            Self::CounterClockwise => 1.0,
        }
    }
}
/// How one biter swoops. Opt-in at spawn; absent means never swoops.
///
/// The roll interval restarts when a dive finishes, so a long pass does not
/// return to a lottery that is already due. The exit also keeps a decaying
/// speed boost for a short recovery window, which lets the biter settle back
/// into its circle instead of snapping from the dive speed to normal.
#[derive(Component, Clone, Debug)]
pub struct Swoop {
    /// Probability of a dive on each roll, `0.0..=1.0`.
    pub chance: f32,
    /// Seconds between rolls while circling or keeping distance. A completed
    /// dive starts the next interval when the dive finishes.
    pub interval: f32,
    pub damage: f32,
    pub speed_boost: f32,
    pub exit_turn: f32,
    pub(crate) next_roll_at: f32,
    pub(crate) called: bool,
    pass: Option<SwoopPass>,
    pub(crate) started_at: f32,
    bitten: bool,
    recovery_until: f32,
}
impl Swoop {
    /// Rolls `chance` every `interval` seconds for `damage` on contact.
    pub fn new(chance: f32, interval: f32, damage: f32) -> Self {
        Self {
            chance,
            interval,
            damage,
            speed_boost: SWOOP_SPEED_BOOST,
            exit_turn: SWOOP_EXIT_TURN,
            next_roll_at: 0.0,
            called: false,
            pass: None,
            started_at: 0.0,
            bitten: false,
            recovery_until: 0.0,
        }
    }
    pub fn is_active(&self) -> bool {
        // The latch alone counts: a pass may never be built when the bearing
        // collapses, and gating on the pass would leave the call with no
        // timeout owner.
        self.called
    }
    /// Ends the dive, starts the post-dive roll interval, and opens the exit
    /// recovery window for dives that actually flew.
    pub(crate) fn finish(&mut self, now: f32) {
        let dove = self.pass.is_some();
        if self.called {
            self.next_roll_at = now + self.interval;
        }
        if dove {
            self.recovery_until = now + SWOOP_RECOVERY_TIME;
        }
        self.called = false;
        self.pass = None;
        self.bitten = false;
    }
    /// Extra speed still in effect while the biter settles out of a dive.
    /// Smoothstep keeps the first frames near full dive speed and lands on
    /// exactly `1.0` at the end of the recovery window.
    pub(crate) fn recovery_boost(&self, now: f32) -> f32 {
        if now >= self.recovery_until {
            return 1.0;
        }
        let remaining = ((self.recovery_until - now) / SWOOP_RECOVERY_TIME).clamp(0.0, 1.0);
        let eased = remaining * remaining * (3.0 - 2.0 * remaining);
        1.0 + self.speed_boost * eased
    }

    pub(crate) fn bite(&mut self) -> bool {
        let available = !self.bitten;
        self.bitten = true;
        available
    }
    pub(crate) fn cleared(&self, position: Vec2, target_distance: f32) -> bool {
        self.pass
            .as_ref()
            .is_some_and(|pass| pass.cleared(position, target_distance))
    }
    pub(crate) fn timed_out(&self, now: f32) -> bool {
        now - self.started_at > SWOOP_TIMEOUT
    }
}
const SWOOP_SPEED_BOOST: f32 = 1.0;
const SWOOP_EXIT_TURN: f32 = 2.0 * std::f32::consts::FRAC_PI_3;
const SWOOP_LOOKAHEAD: f32 = 40.0;
const SWOOP_TRACK_LIMIT: f32 = 0.25;
const SWOOP_TRACK_RATIO: f32 = 0.5;
const SWOOP_ARC_SAMPLES: usize = 12;
/// Seconds over which the dive's speed boost fades after the pass ends.
const SWOOP_RECOVERY_TIME: f32 = 0.4;
pub(crate) const SWOOP_TIMEOUT: f32 = 5.0;

/// One dive's committed flight path through a target snapshot and back out.
#[derive(Clone, Copy, Debug)]
struct SwoopPass {
    start: Vec2,
    control: Vec2,
    end: Vec2,
    target: Vec2,
    origin_target: Vec2,
    exit_bearing: Vec2,
    entry_bearing: Vec2,
    exit_radius: f32,
    length: f32,
    progress: f32,
}

impl SwoopPass {
    fn new(start: Vec2, target: Vec2, exit_radius: f32, heading: Vec2, exit_turn: f32) -> Self {
        let entry_bearing = (start - target).normalize_or_zero();
        // A zero bearing means the biter sits on the target: there is no arc
        // to fly, and the pass collapses to a point that ends at once.
        let orbit = entry_bearing.perp().dot(heading);
        let side = if orbit > 0.0 { -1.0 } else { 1.0 };
        let exit_bearing = entry_bearing.rotate(Vec2::from_angle(side * exit_turn));
        let mut pass = Self {
            start,
            control: Vec2::ZERO,
            end: Vec2::ZERO,
            target,
            origin_target: target,
            exit_bearing,
            entry_bearing,
            exit_radius,
            length: 0.0,
            progress: 0.0,
        };
        pass.rebuild();
        pass
    }

    fn rebuild(&mut self) {
        self.end = self.target + self.exit_bearing * self.exit_radius;
        // Placing the control point here puts the arc's midpoint exactly on
        // the aim point, which is what keeps the pass aimed through it.
        self.control = self.target * 2.0 - (self.start + self.end) * 0.5;
        self.length = self.estimate_length();
    }

    fn track(&mut self, live_target: Vec2, max_step: f32) {
        let limit = self.exit_radius * SWOOP_TRACK_LIMIT;
        let desired =
            self.origin_target + (live_target - self.origin_target).clamp_length_max(limit);
        let step = (desired - self.target).clamp_length_max(max_step);
        if step.length_squared() <= f32::EPSILON {
            return;
        }
        self.target += step;
        self.rebuild();
    }

    fn point(&self, t: f32) -> Vec2 {
        let t = t.clamp(0.0, 1.0);
        let inverse = 1.0 - t;
        self.start * (inverse * inverse) + self.control * (2.0 * inverse * t) + self.end * (t * t)
    }

    fn estimate_length(&self) -> f32 {
        (1..=SWOOP_ARC_SAMPLES)
            .map(|step| {
                let t0 = (step - 1) as f32 / SWOOP_ARC_SAMPLES as f32;
                let t1 = step as f32 / SWOOP_ARC_SAMPLES as f32;
                self.point(t0).distance(self.point(t1))
            })
            .sum()
    }

    /// Advances the arc by `distance` world units, clamped to its end.
    fn advance(&mut self, distance: f32) {
        if self.length > f32::EPSILON {
            self.progress = (self.progress + distance / self.length).clamp(0.0, 1.0);
        } else {
            self.progress = 1.0;
        }
    }

    /// Point `distance` world units further along the arc from the current
    /// progress, clamped to the end.
    fn lead(&self, distance: f32) -> Vec2 {
        let lead = if self.length > f32::EPSILON {
            self.progress + distance / self.length
        } else {
            1.0
        };
        self.point(lead)
    }

    /// Whether the biter has crossed to the far side of the target snapshot
    /// and opened back out to the exit radius, or flown the whole arc.
    fn cleared(&self, position: Vec2, target_distance: f32) -> bool {
        if self.progress >= 1.0 {
            return true;
        }
        let passed = (position - self.target).dot(self.entry_bearing) < 0.0;
        passed && target_distance >= self.exit_radius
    }
}
/// What calls one committed straight dash. Execution is shared; only the
/// trigger differs.
#[derive(Clone, Copy, Debug)]
pub enum DashPolicy {
    /// Break away from the target along a fixed bearing after a swing.
    Breakaway { chance: f32, spread: f32 },
    /// Hop to a fresh tangent after a completed volley.
    VolleyHop { close_range: f32 },
    /// Escape straight out when the target crosses the danger ring.
    Escape { danger_range: f32, cooldown: f32 },
}

impl DashPolicy {
    /// Mode that executes a dash called under this policy.
    pub(crate) fn mode(self) -> MotionMode {
        match self {
            Self::Breakaway { .. } => MotionMode::Kite,
            Self::VolleyHop { .. } => MotionMode::Burst,
            Self::Escape { .. } => MotionMode::Slide,
        }
    }

    /// Seconds after a spent run before another can be called.
    fn cooldown(self) -> f32 {
        match self {
            Self::Escape { cooldown, .. } => cooldown,
            _ => 0.0,
        }
    }

    /// Trigger option arming a dash under this policy, scored by the driver.
    pub(crate) fn trigger(self) -> &'static ScoredOption<()> {
        match self {
            Self::Breakaway { .. } => &TRIGGER_BREAKAWAY,
            Self::VolleyHop { .. } => &TRIGGER_VOLLEY_HOP,
            Self::Escape { .. } => &TRIGGER_ESCAPE,
        }
    }

    /// Chance roll per passed trigger; only breakaways sometimes sit out.
    fn call_chance(self) -> f32 {
        match self {
            Self::Breakaway { chance, .. } => chance,
            Self::VolleyHop { .. } | Self::Escape { .. } => 1.0,
        }
    }

    /// Aim shared by every call under this policy.
    fn aim(self) -> AimKind {
        match self {
            Self::Breakaway { spread, .. } => AimKind::AwayCone { spread },
            Self::VolleyHop { close_range } => AimKind::VolleyTangent { close_range },
            Self::Escape { .. } => AimKind::DirectlyAway,
        }
    }

    /// Incumbent to hand back when the latch is gone but the mode holds.
    pub(crate) fn fallback_mode(self) -> Option<MotionMode> {
        match self {
            Self::Breakaway { .. } => Some(MotionMode::Chase),
            Self::VolleyHop { .. } | Self::Escape { .. } => None,
        }
    }

    /// Target distance calling an escape dash, when this policy escapes.
    pub(crate) fn trigger_range(self) -> Option<f32> {
        match self {
            Self::Escape { danger_range, .. } => Some(danger_range),
            _ => None,
        }
    }
}

/// One committed straight run along a bearing fixed at the call. Opt-in at
/// spawn; absent means the entity never dashes.
///
/// A break fixes its bearing once, so the warrior runs a straight line out and
/// then turns back instead of curving around its target. A volley hop fixes a
/// fresh tangent so the mage keeps its new firing angle. An escape runs
/// straight out on a speed boost so it reads as committed rather than another
/// spacing adjustment.
#[derive(Component, Clone, Debug)]
pub struct Dash {
    /// Shortest and longest run of one dash, in world units.
    pub min_run: f32,
    pub max_run: f32,
    /// Multiple of max speed while dashing.
    pub speed_boost: f32,
    pub policy: DashPolicy,
    run: StraightRun,
    rearm_at: f32,
    last_side: f32,
}

impl Dash {
    /// Breaks away on `chance` of triggers for a rolled `min_run..max_run` run.
    /// The chance is rolled once per completed swing, which keeps repeat
    /// swings without a break rare.
    pub fn breakaway(chance: f32, min_run: f32, max_run: f32) -> Self {
        Self {
            min_run,
            max_run,
            speed_boost: 1.0,
            policy: DashPolicy::Breakaway {
                chance,
                spread: KITE_SPREAD,
            },
            run: StraightRun::default(),
            rearm_at: 0.0,
            last_side: 1.0,
        }
    }

    /// Hops `min_run..max_run` on every completed cast.
    pub fn volley(min_run: f32, max_run: f32, speed_boost: f32, close_range: f32) -> Self {
        Self {
            min_run,
            max_run,
            speed_boost,
            policy: DashPolicy::VolleyHop { close_range },
            run: StraightRun::default(),
            rearm_at: 0.0,
            last_side: 1.0,
        }
    }

    /// Slides `min_run..max_run` whenever a target crosses `danger_range`,
    /// rearming `cooldown` seconds after the last slide ends.
    pub fn escape(
        min_run: f32,
        max_run: f32,
        speed_boost: f32,
        danger_range: f32,
        cooldown: f32,
    ) -> Self {
        Self {
            min_run,
            max_run,
            speed_boost,
            policy: DashPolicy::Escape {
                danger_range,
                cooldown,
            },
            run: StraightRun::default(),
            rearm_at: 0.0,
            last_side: 1.0,
        }
    }

    pub(crate) fn is_called(&self) -> bool {
        self.run.called
    }

    pub(crate) fn ready(&self, now: f32) -> bool {
        now >= self.rearm_at
    }

    pub(crate) fn call(&mut self, direction: Vec2, run: f32) {
        self.run.call(direction, run);
    }

    /// Clears a dash that lost its mode or its target. Only a spent run arms
    /// the cooldown (see `step`): a call that never executed must not lock out
    /// the next escape.
    pub(crate) fn finish(&mut self) {
        self.run.finish();
    }

    /// Routes the latched call into its per-kind context flag.
    pub(crate) fn report_called(&self, context: &mut DecisionContext) {
        let called = self.is_called();
        match self.policy {
            DashPolicy::Breakaway { .. } => context.kite_called = called,
            DashPolicy::VolleyHop { .. } => context.burst_called = called,
            DashPolicy::Escape { .. } => context.slide_called = called,
        }
    }

    /// A fresh tangent that cuts across the target's orbit when it circles, or
    /// flips sides when it holds still. A pinned target inside `close_range`
    /// gets the step-in spread instead of another sidestep.
    pub(crate) fn volley_direction(
        &mut self,
        target: &TargetView,
        close_range: f32,
        brain: &mut Brain,
    ) -> Vec2 {
        if target.distance <= close_range
            && target.velocity.length_squared() <= TARGET_MOVING_SPEED * TARGET_MOVING_SPEED
        {
            let spread = brain.range(-BURST_STEP_IN_SPREAD, BURST_STEP_IN_SPREAD);
            return target.bearing.rotate(Vec2::from_angle(spread));
        }
        let side = if target.tangential_velocity.abs() > TARGET_MOVING_SPEED {
            -target.tangential_velocity.signum()
        } else {
            -self.last_side
        };
        self.last_side = side;
        let tangent = target.bearing.perp() * side;
        (tangent + target.bearing * BURST_OUTWARD_BIAS).normalize_or_zero()
    }

    fn step(&mut self, distance: f32, now: f32) -> Option<Vec2> {
        if !self.run.called {
            return None;
        }
        let direction = self.run.step(distance);
        if !self.run.called {
            self.rearm_at = now + self.policy.cooldown();
        }
        direction
    }
}

/// Shared dash aim: the driver picks one from the policy, rolls chance, aims,
/// and latches the run.
#[derive(Clone, Copy, Debug)]
enum AimKind {
    /// Straight away from the target inside a spread cone.
    AwayCone { spread: f32 },
    /// Fresh tangent after a volley, stepping in when the target is pinned.
    VolleyTangent { close_range: f32 },
    /// Straight away from the target's bearing.
    DirectlyAway,
}

impl AimKind {
    fn aim(self, target: TargetView, dash: &mut Dash, brain: &mut Brain) -> Option<Vec2> {
        match self {
            Self::AwayCone { spread } => {
                let away = (-target.bearing).normalize_or_zero();
                if away.length_squared() > f32::EPSILON {
                    Some(away.rotate(Vec2::from_angle(brain.range(-spread, spread))))
                } else {
                    Some(brain.direction())
                }
            }
            Self::VolleyTangent { close_range } => {
                let direction = dash.volley_direction(&target, close_range, brain);
                (direction.length_squared() > f32::EPSILON).then_some(direction)
            }
            Self::DirectlyAway => {
                if target.bearing.length_squared() > f32::EPSILON {
                    Some(-target.bearing)
                } else {
                    Some(brain.direction())
                }
            }
        }
    }
}

/// Arms one dash call off its scored trigger: a passing trigger plus the
/// policy's chance roll fixes a bearing and latches the run. Losing the
/// target cancels the call so a stale latch cannot re-arm it.
pub(super) fn update_dash_call(dash: &mut Dash, context: &DecisionContext, brain: &mut Brain) {
    if dash.is_called() && context.target.is_none() {
        dash.finish();
    }
    let chance = dash.policy.call_chance();
    if score_option(dash.policy.trigger(), context, false).is_some_and(|score| score > 0.0)
        && (chance >= 1.0 || brain.range(0.0, 1.0) < chance)
    {
        let Some(target) = context.target else {
            return;
        };
        if let Some(direction) = dash.policy.aim().aim(target, dash, brain) {
            dash.call(direction, brain.range(dash.min_run, dash.max_run));
        }
    }
}

/// Rolls the swoop lottery off its scored trigger while circling or keeping
/// distance. A miss schedules the next roll; a hit latches the call and leaves
/// the timer for `finish` to restart after the dive.
pub(super) fn update_swoop_lottery(
    swoop: &mut Swoop,
    context: &DecisionContext,
    mode: MotionMode,
    now: f32,
    brain: &mut Brain,
) {
    if !matches!(mode, MotionMode::Strafe | MotionMode::KeepDistance(_)) {
        return;
    }
    if score_option(&TRIGGER_SWOOP, context, false).is_some_and(|score| score > 0.0) {
        if brain.range(0.0, 1.0) < swoop.chance {
            swoop.called = true;
            swoop.started_at = now;
        } else {
            swoop.next_roll_at = now + swoop.interval;
        }
    }
}

/// One committed straight run along a bearing fixed at the call.
#[derive(Clone, Copy, Debug, Default)]
struct StraightRun {
    direction: Vec2,
    remaining: f32,
    called: bool,
}

impl StraightRun {
    fn call(&mut self, direction: Vec2, run: f32) {
        self.direction = direction;
        self.remaining = run.max(0.0);
        self.called = true;
    }

    fn finish(&mut self) {
        self.called = false;
        self.remaining = 0.0;
    }

    fn step(&mut self, distance: f32) -> Option<Vec2> {
        if !self.called {
            return None;
        }
        if distance >= self.remaining {
            self.finish();
            return None;
        }
        self.remaining -= distance;
        Some(self.direction)
    }
}
/// Brief plant while a caster's equipment is in use. Opt-in at spawn; absent
/// means the character keeps moving through its cast.
///
/// The plant is measured from the start of the attack, so a cast shorter than
/// `hold` leaves a readable recovery beat instead of sliding straight into the
/// next move.
#[derive(Component, Clone, Copy, Debug)]
pub struct CastPlant {
    pub hold: f32,
    hold_until: f32,
    was_attacking: bool,
}

impl CastPlant {
    /// Plants for `hold` seconds starting when the caster begins an attack.
    pub fn new(hold: f32) -> Self {
        Self {
            hold,
            hold_until: 0.0,
            was_attacking: false,
        }
    }

    /// Returns whether the caster should stay planted this frame.
    fn update(&mut self, attacking: bool, now: f32) -> bool {
        if attacking && !self.was_attacking {
            self.hold_until = now + self.hold;
        }
        self.was_attacking = attacking;
        attacking || now < self.hold_until
    }
}
fn pick_wander_direction(
    random: &mut Brain,
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

/// Post-dive speed boost and whether the recovery window is still open.
fn swoop_recovery(swoop: Option<&Swoop>, now: f32) -> (f32, bool) {
    let boost = swoop.map_or(1.0, |swoop| swoop.recovery_boost(now));
    (boost, boost > 1.0)
}

pub(super) fn drive_motion(
    time: Res<Time<Virtual>>,
    ai_seed: Res<AiSeed>,
    mut characters: Query<
        (
            Entity,
            &mut Brain,
            &mut SimpleMotion,
            &Transform,
            Option<&TargetInfo>,
            Has<Targeting>,
            Option<&Anchor>,
            Option<&mut Wander>,
            Option<&mut Strafe>,
            Option<&mut Swoop>,
            Option<&TurnRate>,
            (
                Option<&AttackState>,
                Option<&mut CastPlant>,
                Option<&mut Dash>,
            ),
        ),
    >,
) {
    let now = time.elapsed_secs();

    for (
        entity,
        mut brain,
        mut motion,
        transform,
        target_info,
        has_target,
        anchor,
        mut wander,
        mut strafe,
        mut swoop,
        turn_rate,
        (attack_state, mut cast_plant, mut dash),
    ) in &mut characters
    {
        brain.ensure_seeded(entity, ai_seed.0);
        let delta_secs = time.delta_secs();
        let turn_cap = turn_rate.map(|rate| rate.0 * delta_secs);
        // Shared steering input, read before any arm borrows motion: the arms
        // that steer toward a world point read it here.
        let position = transform.translation.xy();
        // A dive's speed boost fades after the pass, and the orbit holds still
        // during that window so the entry back into the circle does not also
        // flip the tangent.
        let (recovery_boost, recovering) = swoop_recovery(swoop.as_deref(), now);

        // Casters plant through the read of their own wind-up. Keeping this
        // ahead of the steering match also means a committed burst or slide
        // cannot drag the caster out from under a staff that is already firing.
        let attacking = attack_state.is_some_and(|state| state.is_attacking);
        if cast_plant
            .as_deref_mut()
            .is_some_and(|plant| plant.update(attacking, now))
        {
            if !motion.is_idle() {
                motion.stop_moving();
            }
            continue;
        }

        if brain.mode.needs_target() && (!has_target || target_info.is_none()) {
            if !motion.is_idle() {
                motion.stop_moving();
            }
            continue;
        }

        match brain.mode {
            MotionMode::Idle | MotionMode::Hold => {
                if !motion.is_idle() {
                    motion.stop_moving();
                }
            }
            MotionMode::Chase => {
                drive(&mut motion, chase_direction(target_info), Effort::Scaled(1.0), turn_cap);
            }
             // Still in the fight: run from the target, not toward home.
             // Going home happens only once the target is lost.
            MotionMode::Flee => {
                drive(&mut motion, flee_direction(target_info), Effort::Scaled(1.0), turn_cap);
            }
            // Breaks, volley hops and escapes share one straight-run contract
            // and differ only in what calls them; see `DashPolicy`.
            MotionMode::Kite | MotionMode::Burst | MotionMode::Slide => {
                let speed_boost = dash.as_deref().map_or(1.0, |dash| dash.speed_boost);
                let step = run_step(&motion, speed_boost, delta_secs);
                let direction = dash
                    .as_deref_mut()
                    .and_then(|dash| dash.step(step, now));
                drive(&mut motion, direction, Effort::Boosted(speed_boost), turn_cap);
            }
            MotionMode::ReturnToAnchor => drive(
                &mut motion,
                anchor_direction(position, anchor),
                Effort::Scaled(1.0),
                turn_cap,
            ),
            MotionMode::KeepDistance(band) => drive(
                &mut motion,
                keep_distance_direction(target_info, band, wander.as_deref_mut(), now, &mut *brain),
                Effort::Boosted(recovery_boost),
                turn_cap,
            ),
            MotionMode::Strafe => drive(
                &mut motion,
                strafe_direction(
                    target_info,
                    strafe.as_deref_mut(),
                    &mut *brain,
                    now,
                    recovering,
                ),
                Effort::Boosted(recovery_boost),
                turn_cap,
            ),
             MotionMode::Swoop => {
                 let heading = motion.direction;
                 let speed_boost = if recovering {
                     recovery_boost
                 } else {
                     swoop
                         .as_deref()
                         .map_or(1.0, |swoop| 1.0 + swoop.speed_boost)
                 };
                 // Stunned biters hold their place on the arc; the safety bound
                 // finishes the dive once they recover or never do.
                let flight_distance = run_step(&motion, speed_boost, delta_secs);
                 let direction = swoop_direction(
                     target_info,
                     position,
                     heading,
                     swoop.as_deref_mut(),
                     flight_distance,
                     now,
                 );
                 // The bite applier can finish the pass in Execute, before the
                 // next decision changes the mode. Start folding into the circle
                 // on that same frame instead of stopping until the decision.
                 let direction = if direction.is_none() && recovering {
                    strafe_direction(target_info, strafe.as_deref_mut(), &mut *brain, now, true)
                 } else {
                     direction
                 };
                drive(&mut motion, direction, Effort::Boosted(speed_boost), turn_cap);
             }
            MotionMode::Wander => {
                let speed_scale = wander
                    .as_deref()
                    .map_or(WANDER_SPEED_SCALE, |w| w.speed_scale);
                drive(
                    &mut motion,
                    wander_direction(wander.as_deref_mut(), anchor, position, now, &mut *brain),
                    Effort::Scaled(speed_scale),
                    turn_cap,
                );
            }
        }
    }
}

/// How hard to push along the steered heading: a fraction of max speed for
/// gaits, a multiple for committed bursts.
#[derive(Clone, Copy)]
enum Effort {
    Scaled(f32),
    Boosted(f32),
}

fn drive(
    motion: &mut SimpleMotion,
    direction: Option<Vec2>,
    effort: Effort,
    turn_cap: Option<f32>,
) {
    let Some(desired) = direction else {
        if !motion.is_idle() {
            motion.stop_moving();
        }
        return;
    };
    let heading = turn_towards(motion, desired, turn_cap);
    match effort {
        Effort::Scaled(scale) => motion.start_moving_scaled(heading, scale),
        Effort::Boosted(speed_boost) => motion.start_moving_boosted(heading, speed_boost),
    }
}

/// World units one straight-run steering step may consume this frame.
/// Stunned bodies hold still so committed passes neither advance nor finish.
fn run_step(motion: &SimpleMotion, speed_boost: f32, delta_secs: f32) -> f32 {
    if motion.is_stunned() {
        0.0
    } else {
        motion.max_speed * speed_boost * motion.speed_scale() * delta_secs
    }
}
fn turn_towards(motion: &SimpleMotion, desired: Vec2, turn_cap: Option<f32>) -> Vec2 {
    if let Some(max_angle) = turn_cap
        && motion.direction.length_squared() > f32::EPSILON
        && desired.length_squared() > f32::EPSILON
    {
        motion.direction.rotate_towards(desired, max_angle)
    } else {
        desired
    }
}
fn chase_direction(target_info: Option<&TargetInfo>) -> Option<Vec2> {
    let direction = target_info.map_or(Vec2::ZERO, |info| info.direction);
    (direction.length_squared() > f32::EPSILON).then_some(direction)
}
fn flee_direction(target_info: Option<&TargetInfo>) -> Option<Vec2> {
    chase_direction(target_info).map(|direction| -direction)
}
fn anchor_direction(position: Vec2, anchor: Option<&Anchor>) -> Option<Vec2> {
    let anchor = anchor?;
    let to_anchor = anchor.origin() - position;
    (to_anchor.length() > ANCHOR_ARRIVAL_RADIUS).then_some(to_anchor.normalize_or_zero())
}
fn strafe_direction(
    target_info: Option<&TargetInfo>,
    strafe: Option<&mut Strafe>,
    random: &mut Brain,
    now: f32,
    hold_orbit: bool,
) -> Option<Vec2> {
    let info = target_info?;
    let direction = chase_direction(Some(info))?;
    let strafe = strafe?;
    // Do not reroll the orbit side or standoff while the biter is settling out
    // of a dive; a flip here would reverse the tangent at the handoff.
    let orbit_due = now >= strafe.reroll_at;
    if strafe.orbit.is_none() || (!hold_orbit && orbit_due) {
        let orbit = if strafe.smart_orbit {
            smart_strafe_orbit(info, direction, strafe.orbit, random)
        } else {
            match strafe.orbit {
                Some(orbit) if random.range(0.0, 1.0) < strafe.switch_chance => orbit.flipped(),
                Some(orbit) => orbit,
                None if random.range(0.0, 1.0) < 0.5 => Orbit::Clockwise,
                None => Orbit::CounterClockwise,
            }
        };
        strafe.orbit = Some(orbit);
        if strafe.standoff_range > 0.0 {
            strafe.standoff = random.range(-strafe.standoff_range, strafe.standoff_range);
        }
        strafe.reroll_at = now + random.range(strafe.reroll_min, strafe.reroll_max);
    } else if hold_orbit && orbit_due {
        // A dive can outlive the orbit's reroll clock. Start a fresh window at
        // the handoff instead of flipping immediately after the recovery.
        strafe.reroll_at = now + random.range(strafe.reroll_min, strafe.reroll_max);
    }
    let tangent = direction.perp() * strafe.orbit.unwrap_or_default().sign();
    // Positive error means too far from the shifted ring, so the correction
    // points at the target.
    let radial =
        direction * strafe.band.center_error(info.distance - strafe.standoff) * strafe.radial_gain;
    Some((tangent + radial).normalize_or_zero())
}

/// Picks the orbit side that moves against the target's tangential motion.
/// Radial or still targets keep the current side so the mage does not jitter.
fn smart_strafe_orbit(
    info: &TargetInfo,
    direction: Vec2,
    current: Option<Orbit>,
    random: &mut Brain,
) -> Orbit {
    let tangential_motion = info.target_velocity.dot(direction.perp());
    if tangential_motion.abs() > SMART_STRAFE_INPUT_DEADBAND {
        if tangential_motion > 0.0 {
            Orbit::Clockwise
        } else {
            Orbit::CounterClockwise
        }
    } else if let Some(orbit) = current {
        orbit
    } else if random.range(0.0, 1.0) < 0.5 {
        Orbit::Clockwise
    } else {
        Orbit::CounterClockwise
    }
}
fn swoop_direction(
    target_info: Option<&TargetInfo>,
    position: Vec2,
    heading: Vec2,
    swoop: Option<&mut Swoop>,
    flight_distance: f32,
    now: f32,
) -> Option<Vec2> {
    let info = target_info?;
    let swoop = swoop?;
    // Only a non-finite distance blocks the pass: a collapsed (zero-bearing)
    // snapshot builds the zero-length arc `SwoopPass::new` already handles,
    // which clears on the next bite tick instead of stranding the latch with
    // no timeout owner.
    if !info.distance.is_finite() || !swoop.called {
        return None;
    }
    let live_target = position + info.direction * info.distance;
    if swoop.pass.is_none() {
        swoop.pass = Some(SwoopPass::new(
            position,
            live_target,
            info.distance,
            heading,
            swoop.exit_turn,
        ));
        swoop.started_at = now;
        swoop.bitten = false;
    }
    let pass = swoop.pass.as_mut()?;
    pass.advance(flight_distance);
    pass.track(live_target, flight_distance * SWOOP_TRACK_RATIO);
    let to_lead = pass.lead(SWOOP_LOOKAHEAD) - position;
    (to_lead.length_squared() > f32::EPSILON).then(|| to_lead.normalize_or_zero())
}
/// Holds the band; a small deadband keeps the arrival from flickering.
fn keep_distance_direction(
    target_info: Option<&TargetInfo>,
    band: DistanceBand,
    wander: Option<&mut Wander>,
    now: f32,
    random: &mut Brain,
) -> Option<Vec2> {
    let distance = target_info.map_or(0.0, |info| info.distance);
    let direction = target_info.map_or(Vec2::ZERO, |info| info.direction);
    if direction.length_squared() <= f32::EPSILON {
        if distance < band.min() {
            let wander = wander?;
            if now >= wander.next_direction_at {
                wander.direction = random.direction();
                wander.next_direction_at = now + 0.2;
            }
            return Some(wander.direction);
        }
        return None;
    }
    let error = band.error(distance);
    if error.abs() < KEEP_ARRIVAL_DEADBAND {
        None
    } else if error < 0.0 {
        Some(-direction)
    } else {
        Some(direction)
    }
}
fn wander_direction(
    wander: Option<&mut Wander>,
    anchor: Option<&Anchor>,
    position: Vec2,
    now: f32,
    random: &mut Brain,
) -> Option<Vec2> {
    let wander = wander?;
    if now >= wander.next_direction_at {
        wander.direction = pick_wander_direction(random, anchor, position);
        wander.next_direction_at = now + random.range(wander.min, wander.max);
    }
    Some(wander.direction)
}
