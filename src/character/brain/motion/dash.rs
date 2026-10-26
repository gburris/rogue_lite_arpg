//! Committed straight runs: the warrior's kite, the fire mage's hop and the ice mage's slide.

use bevy::prelude::*;

use super::strafe::Orbit;
use super::{MotionContext, MotionExecutor, MotionStep, RunTag, SteeringEffort};
use crate::character::brain::senses::build_target_view;
use crate::character::brain::{Motion, TARGET_MOVING_SPEED, TargetView, Tuning};

const KITE_SPREAD: f32 = std::f32::consts::FRAC_PI_4;

const HOP_STEP_IN_SPREAD: f32 = 0.35;
const HOP_OUTWARD_BIAS: f32 = 0.15;

/// What a dash is for, which sets its tuning and aim.
#[derive(Clone, Copy, Debug)]
pub enum DashPolicy {
    Kite { chance: f32, spread: f32 },
    Hop { close_range: f32 },
    Slide { danger_range: f32, cooldown: f32 },
}

impl DashPolicy {
    fn name(self) -> &'static str {
        match self {
            Self::Kite { .. } => "Kite",
            Self::Hop { .. } => "Hop",
            Self::Slide { .. } => "Slide",
        }
    }

    fn tuning(self) -> Tuning {
        match self {
            Self::Kite { chance, .. } => Tuning {
                chance,
                ..Tuning::default()
            },
            Self::Hop { .. } => Tuning::default(),
            Self::Slide {
                danger_range,
                cooldown,
            } => Tuning {
                reach: Some(danger_range),
                cooldown,
                ..Tuning::default()
            },
        }
    }

    fn aim(self) -> AimKind {
        match self {
            Self::Kite { spread, .. } => AimKind::Away { spread },
            Self::Hop { close_range } => AimKind::HopTangent { close_range },
            Self::Slide { .. } => AimKind::Away { spread: 0.0 },
        }
    }
}

/// A committed straight run along a direction fixed at its start. Opt-in at spawn.
#[derive(Component, Clone, Debug)]
pub struct Dash {
    pub min_run: f32,
    pub max_run: f32,
    pub speed_boost: f32,
    policy: DashPolicy,
    run: StraightRun,
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
            tag: RunTag::default(),
        }
    }

    pub fn kite(chance: f32, min_run: f32, max_run: f32) -> Self {
        Self::with_policy(
            min_run,
            max_run,
            1.0,
            DashPolicy::Kite {
                chance,
                spread: KITE_SPREAD,
            },
        )
    }

    pub fn hop(min_run: f32, max_run: f32, speed_boost: f32, close_range: f32) -> Self {
        Self::with_policy(
            min_run,
            max_run,
            speed_boost,
            DashPolicy::Hop { close_range },
        )
    }

    pub fn slide(
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
            DashPolicy::Slide {
                danger_range,
                cooldown,
            },
        )
    }

    pub fn name(&self) -> &'static str {
        self.policy.name()
    }
}

fn hop_direction(target: &TargetView, close_range: f32, context: &mut MotionContext) -> Vec2 {
    if target.distance <= close_range
        && target.velocity.length_squared() <= TARGET_MOVING_SPEED * TARGET_MOVING_SPEED
    {
        let spread = context.rng.range(-HOP_STEP_IN_SPREAD, HOP_STEP_IN_SPREAD);
        return target.bearing.rotate(Vec2::from_angle(spread));
    }
    let now = context.input.now;
    let orbit = match context.strafe.as_deref_mut() {
        Some(strafe) => strafe.flip_orbit(now, context.rng),
        None => Orbit::random(context.rng),
    };
    let tangent = target.bearing.perp() * orbit.sign();
    (tangent - target.bearing * HOP_OUTWARD_BIAS).normalize_or_zero()
}

impl MotionExecutor for Dash {
    fn motion(&self) -> Motion {
        Motion::Dash
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

    fn step(&mut self, context: &mut MotionContext) -> MotionStep {
        if self.tag.spent {
            return MotionStep::Spent;
        }
        if !self.run.called {
            let policy = self.policy;
            let aimed = build_target_view(true, context.input.target_info)
                .and_then(|target| policy.aim().aim(target, context));
            let Some(direction) = aimed else {
                self.tag.spent = true;
                return MotionStep::Aborted;
            };
            let run = context.rng.range(self.min_run, self.max_run);
            self.run.call(direction, run);
        }
        let distance = context.distance_this_frame(self.speed_boost);
        match self.run.step(distance) {
            Some(direction) => MotionStep::Steer {
                direction: Some(direction),
                effort: SteeringEffort::Boosted(self.speed_boost),
            },
            None => {
                self.tag.spent = true;
                MotionStep::Finished
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum AimKind {
    Away { spread: f32 },
    HopTangent { close_range: f32 },
}

impl AimKind {
    fn aim(self, target: TargetView, context: &mut MotionContext) -> Option<Vec2> {
        let rng = &mut *context.rng;
        match self {
            Self::Away { spread } => {
                let away = (-target.bearing).normalize_or_zero();
                if away.length_squared() > f32::EPSILON {
                    Some(away.rotate(Vec2::from_angle(rng.range(-spread, spread))))
                } else {
                    Some(rng.direction())
                }
            }
            Self::HopTangent { close_range } => {
                let direction = hop_direction(&target, close_range, context);
                (direction.length_squared() > f32::EPSILON).then_some(direction)
            }
        }
    }
}

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
