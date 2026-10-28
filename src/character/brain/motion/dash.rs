//! Committed straight runs: the warrior's kite, the fire mage's hop and the ice mage's slide.

use bevy::prelude::*;

use super::{
    Capability, MotionContext, MotionExecutor, MotionStep, RunTag, SpeedBoost, SteeringEffort,
};
use crate::character::brain::perception::{TargetView, build_target_view};
use crate::character::brain::{Motion, Tuning};

/// Speed below which a target counts as holding still rather than orbiting.
const TARGET_MOVING_SPEED: f32 = 20.0;

const KITE_SPREAD: f32 = std::f32::consts::FRAC_PI_4;

const HOP_STEP_IN_SPREAD: f32 = 0.35;
const HOP_OUTWARD_BIAS: f32 = 0.15;

/// A committed straight run along a direction fixed at its start.
#[derive(Component, Clone, Debug)]
pub struct Dash {
    pub min_run: f32,
    pub max_run: f32,
    pub speed_boost: f32,
    pub tuning: Tuning,
    aim: AimKind,
    debug_name: &'static str,
    run: StraightRun,
    tag: RunTag,
}

impl Dash {
    fn with_aim(
        min_run: f32,
        max_run: f32,
        speed_boost: f32,
        tuning: Tuning,
        aim: AimKind,
        debug_name: &'static str,
    ) -> Self {
        Self {
            min_run,
            max_run,
            speed_boost,
            tuning,
            aim,
            debug_name,
            run: StraightRun::default(),
            tag: RunTag::default(),
        }
    }

    pub fn kite(chance: f32, min_run: f32, max_run: f32) -> Self {
        Self::with_aim(
            min_run,
            max_run,
            1.0,
            Tuning {
                chance,
                ..Tuning::default()
            },
            AimKind::Away {
                spread: KITE_SPREAD,
            },
            "Kite",
        )
    }

    pub fn hop(min_run: f32, max_run: f32, speed_boost: f32, close_range: f32) -> Self {
        Self::with_aim(
            min_run,
            max_run,
            speed_boost,
            Tuning::default(),
            AimKind::HopTangent { close_range },
            "Hop",
        )
    }

    pub fn slide(
        min_run: f32,
        max_run: f32,
        speed_boost: f32,
        danger_range: f32,
        cooldown: f32,
    ) -> Self {
        Self::with_aim(
            min_run,
            max_run,
            speed_boost,
            Tuning {
                reach: Some(danger_range),
                cooldown,
                ..Tuning::default()
            },
            AimKind::Away { spread: 0.0 },
            "Slide",
        )
    }

    pub fn name(&self) -> &'static str {
        self.debug_name
    }
}

fn hop_direction(target: &TargetView, close_range: f32, context: &mut MotionContext) -> Vec2 {
    if target.distance <= close_range
        && target.velocity.length_squared() <= TARGET_MOVING_SPEED * TARGET_MOVING_SPEED
    {
        let spread = context.rng.range(-HOP_STEP_IN_SPREAD, HOP_STEP_IN_SPREAD);
        return target.bearing.rotate(Vec2::from_angle(spread));
    }
    // Cut across to the side opposite the current heading, so an orbit that
    // follows reads the new heading and circles back the other way.
    let heading_side = target.bearing.perp().dot(context.input.heading);
    let tangent_sign = if heading_side.abs() > f32::EPSILON {
        -heading_side.signum()
    } else if context.rng.range(0.0, 1.0) < 0.5 {
        1.0
    } else {
        -1.0
    };
    let tangent = target.bearing.perp() * tangent_sign;
    (tangent - target.bearing * HOP_OUTWARD_BIAS).normalize_or_zero()
}

impl Capability for Dash {
    type Key = Motion;

    fn key(&self) -> Motion {
        Motion::Dash
    }

    fn tuning(&self) -> Tuning {
        self.tuning
    }
}

impl MotionExecutor for Dash {
    fn tag(&mut self) -> &mut RunTag {
        &mut self.tag
    }

    fn reset(&mut self, _now: f32) -> Option<SpeedBoost> {
        self.run.finish();
        None
    }

    fn step(&mut self, context: &mut MotionContext) -> MotionStep {
        if !self.run.called {
            // Steering halts a target-needing motion without a target, so this
            // waits rather than aiming blind; the brain drops the behavior.
            let Some(target) =
                build_target_view(true, context.input.target_info, context.input.now)
            else {
                return MotionStep::Steer {
                    direction: None,
                    effort: SteeringEffort::Boosted(self.speed_boost),
                };
            };
            let direction = self.aim.aim(target, context);
            let run = context.rng.range(self.min_run, self.max_run);
            self.run.call(direction, run);
        }
        let distance = context.distance_this_frame(self.speed_boost);
        match self.run.step(distance) {
            Some(direction) => MotionStep::Steer {
                direction: Some(direction),
                effort: SteeringEffort::Boosted(self.speed_boost),
            },
            None => MotionStep::Finished { speed_boost: None },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum AimKind {
    Away { spread: f32 },
    HopTangent { close_range: f32 },
}

impl AimKind {
    fn aim(self, target: TargetView, context: &mut MotionContext) -> Vec2 {
        let direction = match self {
            Self::Away { spread } => (-target.bearing)
                .normalize_or_zero()
                .rotate(Vec2::from_angle(context.rng.range(-spread, spread))),
            Self::HopTangent { close_range } => hop_direction(&target, close_range, context),
        };
        // A zero bearing (the characters overlap) has no direction to aim along.
        if direction.length_squared() > f32::EPSILON {
            direction
        } else {
            context.rng.direction()
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
