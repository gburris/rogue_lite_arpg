//! Committed straight runs: the warrior's kite, the fire mage's burst and the
//! ice mage's slide are all one dash maneuver under different policies.

use bevy::prelude::*;

use super::{Effort, ManeuverContext, ManeuverExecutor, ManeuverStep, RunTag};
use crate::character::brain::senses::target_view;
use crate::character::brain::{BrainRng, Maneuver, TARGET_MOVING_SPEED, TargetView, Tuning};

/// Half-angle of a warrior's break cone around straight away from its target.
const KITE_SPREAD: f32 = std::f32::consts::FRAC_PI_4;

/// Half-angle of the fire mage's step-in spread around the target bearing.
const BURST_STEP_IN_SPREAD: f32 = 0.35;
/// Small outward bias on a tangent burst so a volley does not walk the mage
/// into the target when the band is already tight.
const BURST_OUTWARD_BIAS: f32 = 0.15;

/// What distinguishes one committed straight dash: its tuning and aim.
/// Execution is shared, and what *starts* it is scored by the brain
/// definition's maneuver decisions.
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
    /// One-word label for debug displays.
    fn name(self) -> &'static str {
        match self {
            Self::Breakaway { .. } => "Kite",
            Self::VolleyHop { .. } => "Burst",
            Self::Escape { .. } => "Slide",
        }
    }

    /// Chance, spacing and reach the decision tick applies to this dash.
    fn tuning(self) -> Tuning {
        match self {
            // Only breakaways sometimes sit out a passed trigger.
            Self::Breakaway { chance, .. } => Tuning {
                chance,
                ..Tuning::default()
            },
            Self::VolleyHop { .. } => Tuning::default(),
            Self::Escape {
                danger_range,
                cooldown,
            } => Tuning {
                reach: Some(danger_range),
                cooldown,
                ..Tuning::default()
            },
        }
    }

    /// Aim shared by every start under this policy.
    fn aim(self) -> AimKind {
        match self {
            Self::Breakaway { spread, .. } => AimKind::AwayCone { spread },
            Self::VolleyHop { close_range } => AimKind::VolleyTangent { close_range },
            Self::Escape { .. } => AimKind::DirectlyAway,
        }
    }
}

/// One committed straight run along a bearing fixed at the start. Opt-in at
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
    policy: DashPolicy,
    run: StraightRun,
    last_side: f32,
    tag: RunTag,
}

impl Dash {
    fn with_policy(min_run: f32, max_run: f32, speed_boost: f32, policy: DashPolicy) -> Self {
        Self {
            min_run,
            max_run,
            speed_boost,
            policy,
            run: StraightRun::default(),
            last_side: 1.0,
            tag: RunTag::default(),
        }
    }

    /// Breaks away on `chance` of triggers for a rolled `min_run..max_run` run.
    /// The chance is rolled once per completed swing, which keeps repeat
    /// swings without a break rare.
    pub fn breakaway(chance: f32, min_run: f32, max_run: f32) -> Self {
        Self::with_policy(
            min_run,
            max_run,
            1.0,
            DashPolicy::Breakaway {
                chance,
                spread: KITE_SPREAD,
            },
        )
    }

    /// Hops `min_run..max_run` on every completed cast.
    pub fn volley(min_run: f32, max_run: f32, speed_boost: f32, close_range: f32) -> Self {
        Self::with_policy(
            min_run,
            max_run,
            speed_boost,
            DashPolicy::VolleyHop { close_range },
        )
    }

    /// Slides `min_run..max_run` whenever a target crosses `danger_range`,
    /// allowing another `cooldown` seconds after the last slide ends.
    pub fn escape(
        min_run: f32,
        max_run: f32,
        speed_boost: f32,
        danger_range: f32,
        cooldown: f32,
    ) -> Self {
        Self::with_policy(
            min_run,
            max_run,
            speed_boost,
            DashPolicy::Escape {
                danger_range,
                cooldown,
            },
        )
    }

    /// What this dash is called under its policy, for debug displays.
    pub fn name(&self) -> &'static str {
        self.policy.name()
    }

    /// A fresh tangent that cuts across the target's orbit when it circles, or
    /// flips sides when it holds still. A pinned target inside `close_range`
    /// gets the step-in spread instead of another sidestep.
    fn volley_direction(
        &mut self,
        target: &TargetView,
        close_range: f32,
        brain: &mut BrainRng,
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
}

impl ManeuverExecutor for Dash {
    fn maneuver(&self) -> Maneuver {
        Maneuver::Dash
    }

    fn tuning(&self) -> Tuning {
        self.policy.tuning()
    }

    fn tag(&mut self) -> &mut RunTag {
        &mut self.tag
    }

    fn reset(&mut self, _now: f32) {
        self.run.finish();
    }

    /// Advances the run, aiming and rolling its length on the first step
    /// after the brain starts it.
    fn step(&mut self, context: &mut ManeuverContext) -> ManeuverStep {
        if self.tag.spent {
            return ManeuverStep::Spent;
        }
        if !self.run.called {
            let policy = self.policy;
            let aimed = target_view(true, context.input.target_info)
                .and_then(|target| policy.aim().aim(target, self, context.rng));
            let Some(direction) = aimed else {
                self.tag.spent = true;
                return ManeuverStep::Finished;
            };
            let run = context.rng.range(self.min_run, self.max_run);
            self.run.call(direction, run);
        }
        let distance = context.run_step(self.speed_boost);
        match self.run.step(distance) {
            Some(direction) => ManeuverStep::Steer {
                direction: Some(direction),
                effort: Effort::Boosted(self.speed_boost),
            },
            None => {
                self.tag.spent = true;
                ManeuverStep::Finished
            }
        }
    }
}

/// Shared dash aim: picked from the policy when a dash starts.
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
    fn aim(self, target: TargetView, dash: &mut Dash, brain: &mut BrainRng) -> Option<Vec2> {
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

/// One committed straight run along a bearing fixed at the start.
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
