use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use super::{
    BlockedTimer, CircleDirection, DecayingSpeedBoost, FiniteMotion, MotionKey, MotionProgress,
    MotionStep, RunningMotionOf, SteerInput, Steering,
};

const SWOOP_EXTRA_SPEED: f32 = 1.0;
const SWOOP_EXIT_TURN: f32 = 2.0 * std::f32::consts::FRAC_PI_3;
const SWOOP_LOOKAHEAD: f32 = 40.0;
const SWOOP_TRACK_LIMIT: f32 = 0.25;
const SWOOP_TRACK_RATIO: f32 = 0.5;
const SWOOP_ARC_SAMPLES: usize = 12;
const SWOOP_RECOVERY_TIME: f32 = 0.4;
/// Seconds a swoop may stay blocked before it ends early.
const SWOOP_BLOCKED_SECONDS: f32 = 0.2;

/// A committed curved move through the target and back out. It ends early when blocked or
/// interrupted, and leaves a [`DecayingSpeedBoost`] when it ends.
#[derive(Component, Clone, Copy, Debug)]
#[require(FiniteMotion, MotionKey::Swoop)]
pub struct Swoop {
    extra_speed: f32,
    exit_turn: f32,
}

impl Default for Swoop {
    fn default() -> Self {
        Self {
            extra_speed: SWOOP_EXTRA_SPEED,
            exit_turn: SWOOP_EXIT_TURN,
        }
    }
}

/// A [`Swoop`] in progress, on its running motion. Removing it once the swoop has taken flight,
/// however it ended, gives the character a [`DecayingSpeedBoost`].
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(on_remove = leave_swoop_speed_boost)]
pub(super) struct SwoopProgress {
    path: Option<SwoopPath>,
    blocked_timer: BlockedTimer,
}

impl MotionProgress for SwoopProgress {
    type Tuning = Swoop;

    /// Ends once past the target, or when blocked or the target is lost.
    fn step(&mut self, swoop: &Swoop, input: &SteerInput) -> MotionStep {
        let Some(target) = input.target else {
            return MotionStep::Finished;
        };
        let speed_multiplier = 1.0 + swoop.extra_speed;
        let steer = |direction| MotionStep::Steer(Steering::boosted(direction, speed_multiplier));

        let cleared = self
            .path
            .as_ref()
            .is_some_and(|path| path.cleared(input.position, target.distance));
        let blocked = self.path.is_some()
            && input.tick_blocked_timer(&mut self.blocked_timer, SWOOP_BLOCKED_SECONDS);
        if cleared || blocked {
            return MotionStep::Finished;
        }

        let flight_distance = input.distance_this_frame(speed_multiplier);
        let path = self.path.get_or_insert_with(|| {
            SwoopPath::new(
                input.position,
                target.position,
                target.distance,
                input.heading,
                swoop.exit_turn,
            )
        });
        path.advance(flight_distance);
        path.track(target.position, flight_distance * SWOOP_TRACK_RATIO);

        steer((path.lead(SWOOP_LOOKAHEAD) - input.position).try_normalize())
    }
}

fn leave_swoop_speed_boost(mut world: DeferredWorld, context: HookContext) {
    let Ok((progress, &RunningMotionOf { character }, &Swoop { extra_speed, .. })) = world
        .entity(context.entity)
        .get_components::<(&SwoopProgress, &RunningMotionOf, &Swoop)>()
    else {
        return;
    };
    if progress.path.is_none() {
        return;
    }

    let decaying_speed_boost =
        DecayingSpeedBoost::new(extra_speed, SWOOP_RECOVERY_TIME, EaseFunction::SmoothStep);
    world
        .commands()
        .entity(character)
        .try_insert(decaying_speed_boost);
}

#[derive(Clone, Copy, Debug)]
struct SwoopPath {
    start: Vec2,
    curve: CubicSegment<Vec2>,
    tracked_target_position: Vec2,
    initial_target_position: Vec2,
    exit_bearing: Vec2,
    entry_bearing: Vec2,
    exit_radius: f32,
    length: f32,
    progress: f32,
}

impl SwoopPath {
    fn new(start: Vec2, target: Vec2, exit_radius: f32, heading: Vec2, exit_turn: f32) -> Self {
        let entry_bearing = (start - target).normalize_or_zero();
        let circle_direction = CircleDirection::from_heading(-entry_bearing, heading);
        // The path keeps turning the way the character circles, which swings the exit bearing
        // around the target the other way.
        let exit_turn = circle_direction.reversed().sign_angle(exit_turn);
        let exit_bearing = Rot2::radians(exit_turn) * entry_bearing;
        let mut path = Self {
            start,
            curve: CubicSegment::default(),
            tracked_target_position: target,
            initial_target_position: target,
            exit_bearing,
            entry_bearing,
            exit_radius,
            length: 0.0,
            progress: 0.0,
        };
        path.rebuild();
        path
    }

    fn rebuild(&mut self) {
        let end = self.tracked_target_position + self.exit_bearing * self.exit_radius;
        // Puts the curve's midpoint exactly on the target.
        let control = self.tracked_target_position * 2.0 - (self.start + end) * 0.5;
        self.curve = build_quadratic_bezier(self.start, control, end);
        self.length = self.estimate_length();
    }

    fn track(&mut self, live_target: Vec2, max_step: f32) {
        let limit = self.exit_radius * SWOOP_TRACK_LIMIT;
        let desired = self.initial_target_position
            + (live_target - self.initial_target_position).clamp_length_max(limit);
        let step = (desired - self.tracked_target_position).clamp_length_max(max_step);
        if step.length_squared() <= f32::EPSILON {
            return;
        }
        self.tracked_target_position += step;
        self.rebuild();
    }

    fn point(&self, t: f32) -> Vec2 {
        self.curve.position(t.clamp(0.0, 1.0))
    }

    fn estimate_length(&self) -> f32 {
        let mut length = 0.0;
        let mut previous = self.curve.position(0.0);
        for sample in self.curve.iter_positions(SWOOP_ARC_SAMPLES).skip(1) {
            length += previous.distance(sample);
            previous = sample;
        }
        length
    }

    fn progress_after(&self, distance: f32) -> f32 {
        if self.length > f32::EPSILON {
            self.progress + distance / self.length
        } else {
            1.0
        }
    }

    fn advance(&mut self, distance: f32) {
        self.progress = self.progress_after(distance).clamp(0.0, 1.0);
    }

    fn lead(&self, distance: f32) -> Vec2 {
        self.point(self.progress_after(distance))
    }

    fn cleared(&self, position: Vec2, target_distance: f32) -> bool {
        if self.progress >= 1.0 {
            return true;
        }
        let passed = (position - self.tracked_target_position).dot(self.entry_bearing) < 0.0;
        passed && target_distance >= self.exit_radius
    }
}

/// The quadratic Bézier curve from `start` to `end` pulled toward `control`, raised to the cubic
/// segment that traces it exactly.
fn build_quadratic_bezier(start: Vec2, control: Vec2, end: Vec2) -> CubicSegment<Vec2> {
    // A quadratic leaves each end at twice the offset to `control`, a cubic at three times the
    // offset to its inner point, so inner points 2/3 of the way to `control` trace the same curve.
    CubicSegment::new_bezier([
        start,
        start + (control - start) * (2.0 / 3.0),
        end + (control - end) * (2.0 / 3.0),
        end,
    ])
}
