use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use super::{
    BlockedTimer, DecayingSpeedBoost, Motion, MotionOf, MotionStep, RunningMotion, SteerInput,
    SteeredCharacter, Steering, SteeringTarget, SteeringTargets, TunedBy,
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
#[require(Motion::Swoop)]
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

/// Advances each swoop in progress. Ends it once past the target or when the target is lost.
pub(super) fn apply_swoop(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    targets: SteeringTargets,
    characters: Query<SteeredCharacter>,
    swoops: Query<&Swoop>,
    mut running_swoops: Query<(
        Entity,
        &MotionOf,
        &RunningMotion,
        &TunedBy,
        &mut SwoopProgress,
        &mut Steering,
    )>,
) {
    for (entity, motion_of, running, tuned_by, mut progress, mut steering) in &mut running_swoops {
        let swoop = swoops.get(tuned_by.source).ok();
        let character = characters.get(motion_of.character).ok();
        let input = character
            .as_ref()
            .and_then(|character| character.build_steer_input(running, &targets, &time));
        let step = match (swoop, input) {
            (
                Some(&swoop),
                Some(
                    input @ SteerInput {
                        target: Some(target),
                        ..
                    },
                ),
            ) => progress.step(swoop, &input, &target),
            _ => MotionStep::Finished,
        };

        match step {
            MotionStep::Steer(next) => *steering = next,
            MotionStep::Finished => {
                commands.entity(entity).try_despawn();
            }
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

impl SwoopProgress {
    fn step(&mut self, swoop: Swoop, input: &SteerInput, target: &SteeringTarget) -> MotionStep {
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

        let to_lead = path.lead(SWOOP_LOOKAHEAD) - input.position;
        steer((to_lead.length_squared() > f32::EPSILON).then(|| to_lead.normalize_or_zero()))
    }
}

fn leave_swoop_speed_boost(mut world: DeferredWorld, context: HookContext) {
    let Ok((progress, &MotionOf { character }, &TunedBy { source })) = world
        .entity(context.entity)
        .get_components::<(&SwoopProgress, &MotionOf, &TunedBy)>()
    else {
        return;
    };
    if progress.path.is_none() {
        return;
    }
    let Some(&Swoop { extra_speed, .. }) = world.get(source) else {
        return;
    };

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
    control: Vec2,
    end: Vec2,
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
        let heading_side = entry_bearing.perp().dot(heading);
        let side = if heading_side > 0.0 { -1.0 } else { 1.0 };
        let exit_bearing = entry_bearing.rotate(Vec2::from_angle(side * exit_turn));
        let mut path = Self {
            start,
            control: Vec2::ZERO,
            end: Vec2::ZERO,
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
        self.end = self.tracked_target_position + self.exit_bearing * self.exit_radius;
        // Puts the curve's midpoint exactly on the target.
        self.control = self.tracked_target_position * 2.0 - (self.start + self.end) * 0.5;
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
        let passed = (position - self.tracked_target_position).dot(self.entry_bearing) < 0.0;
        passed && target_distance >= self.exit_radius
    }
}
