//! The bat's dive through the target and back out.

use bevy::prelude::*;

use super::{
    Capability, MotionContext, MotionExecutor, MotionStep, RunTag, SpeedBoost, SteeringEffort,
};
use crate::character::brain::{Motion, Tuning};

const SWOOP_SPEED_BOOST: f32 = 1.0;
const SWOOP_EXIT_TURN: f32 = 2.0 * std::f32::consts::FRAC_PI_3;
const SWOOP_LOOKAHEAD: f32 = 40.0;
const SWOOP_TRACK_LIMIT: f32 = 0.25;
const SWOOP_TRACK_RATIO: f32 = 0.5;
const SWOOP_ARC_SAMPLES: usize = 12;
const SWOOP_RECOVERY_TIME: f32 = 0.4;

/// A committed curved dive through the target; on ending it hands steering a
/// decaying [`SpeedBoost`]. Opt-in at spawn.
#[derive(Component, Clone, Debug)]
pub struct Swoop {
    pub tuning: Tuning,
    pub speed_boost: f32,
    pub exit_turn: f32,
    pass: Option<SwoopPass>,
    tag: RunTag,
}

impl Swoop {
    /// Dives at targets within `reach`, rolling `chance` each time it wins and
    /// waiting `cooldown` seconds after a dive ends.
    pub fn new(chance: f32, reach: f32, cooldown: f32) -> Self {
        Self {
            tuning: Tuning {
                reach: Some(reach),
                chance,
                cooldown,
            },
            speed_boost: SWOOP_SPEED_BOOST,
            exit_turn: SWOOP_EXIT_TURN,
            pass: None,
            tag: RunTag::default(),
        }
    }

    fn finish(&mut self, now: f32) -> Option<SpeedBoost> {
        // Cleared or cut short, a pass hands its boost to steering to ease off.
        self.pass.take().map(|_| {
            SpeedBoost::new(
                self.speed_boost,
                SWOOP_RECOVERY_TIME,
                EaseFunction::SmoothStep,
                now,
            )
        })
    }
}

impl Capability for Swoop {
    type Key = Motion;

    fn key(&self) -> Motion {
        Motion::Swoop
    }

    fn tuning(&self) -> Tuning {
        self.tuning
    }
}

impl MotionExecutor for Swoop {
    fn tag(&mut self) -> &mut RunTag {
        &mut self.tag
    }

    fn reset(&mut self, now: f32) -> Option<SpeedBoost> {
        self.finish(now)
    }

    fn step(&mut self, context: &mut MotionContext) -> MotionStep {
        let input = context.input;
        let now = input.now;
        let speed_boost = 1.0 + self.speed_boost;
        let steer = |direction| MotionStep::Steer {
            direction,
            effort: SteeringEffort::Boosted(speed_boost),
        };
        let Some(info) = input.target_info else {
            return steer(None);
        };
        let cleared = self
            .pass
            .as_ref()
            .is_some_and(|pass| pass.cleared(input.position, info.distance));
        if cleared {
            let speed_boost = self.finish(now);
            return MotionStep::Finished { speed_boost };
        }
        if !info.distance.is_finite() {
            return steer(None);
        }
        let flight_distance = context.distance_this_frame(speed_boost);
        let heading = context.movement.direction;
        let live_target = input.position + info.direction * info.distance;
        let exit_turn = self.exit_turn;
        let pass = self.pass.get_or_insert_with(|| {
            SwoopPass::new(
                input.position,
                live_target,
                info.distance,
                heading,
                exit_turn,
            )
        });
        pass.advance(flight_distance);
        pass.track(live_target, flight_distance * SWOOP_TRACK_RATIO);
        let to_lead = pass.lead(SWOOP_LOOKAHEAD) - input.position;
        steer((to_lead.length_squared() > f32::EPSILON).then(|| to_lead.normalize_or_zero()))
    }
}

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
        let heading_side = entry_bearing.perp().dot(heading);
        let side = if heading_side > 0.0 { -1.0 } else { 1.0 };
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
        // Puts the curve's midpoint exactly on the target.
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

    fn advance(&mut self, distance: f32) {
        if self.length > f32::EPSILON {
            self.progress = (self.progress + distance / self.length).clamp(0.0, 1.0);
        } else {
            self.progress = 1.0;
        }
    }

    fn lead(&self, distance: f32) -> Vec2 {
        let lead = if self.length > f32::EPSILON {
            self.progress + distance / self.length
        } else {
            1.0
        };
        self.point(lead)
    }

    fn cleared(&self, position: Vec2, target_distance: f32) -> bool {
        if self.progress >= 1.0 {
            return true;
        }
        let passed = (position - self.target).dot(self.entry_bearing) < 0.0;
        passed && target_distance >= self.exit_radius
    }
}
