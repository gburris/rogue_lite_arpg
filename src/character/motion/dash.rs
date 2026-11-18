use std::f32::consts::TAU;

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::RngExt;

use super::{
    BlockedTimer, MotionFinished, MotionKind, MotionStep, RunningMotion, SteerInput,
    SteeredCharacter, Steering, SteeringTarget, SteeringTargets, start_motion_source_cooldown,
};
use crate::interval::Interval;

/// Speed below which a target counts as stationary.
const TARGET_MOVING_SPEED: f32 = 20.0;

const KITE_SPREAD: f32 = std::f32::consts::FRAC_PI_4;

const HOP_STEP_IN_SPREAD: f32 = 0.35;
const HOP_OUTWARD_BIAS: f32 = 0.15;

/// Seconds a dash may stay blocked before it ends early.
const DASH_BLOCKED_SECONDS: f32 = 0.15;

/// A committed straight-line move in a direction chosen when it starts. It ends early when blocked
/// or interrupted.
#[derive(Component, Clone, Copy, Debug)]
#[require(MotionKind::Dash)]
pub struct Dash {
    run_distance: Interval,
    speed_multiplier: f32,
    aim_rule: DashAimRule,
    debug_name: &'static str,
}

impl Dash {
    fn new(
        run_distance: Interval,
        speed_multiplier: f32,
        aim_rule: DashAimRule,
        debug_name: &'static str,
    ) -> Self {
        Self {
            run_distance,
            speed_multiplier,
            aim_rule,
            debug_name,
        }
    }

    pub fn kite(run_distance: Interval) -> Self {
        Self::new(
            run_distance,
            1.0,
            DashAimRule::Away {
                spread: KITE_SPREAD,
            },
            "Kite",
        )
    }

    pub fn hop(run_distance: Interval, speed_multiplier: f32, close_range: f32) -> Self {
        Self::new(
            run_distance,
            speed_multiplier,
            DashAimRule::HopTangent { close_range },
            "Hop",
        )
    }

    pub fn slide(run_distance: Interval, speed_multiplier: f32) -> Self {
        Self::new(
            run_distance,
            speed_multiplier,
            DashAimRule::Away { spread: 0.0 },
            "Slide",
        )
    }

    pub fn name(&self) -> &'static str {
        self.debug_name
    }
}

fn hop_direction(target: &SteeringTarget, close_range: f32, heading: Vec2) -> Vec2 {
    if target.distance <= close_range
        && target.velocity.length_squared() <= TARGET_MOVING_SPEED * TARGET_MOVING_SPEED
    {
        let spread = rand::rng().random_range(-HOP_STEP_IN_SPREAD..=HOP_STEP_IN_SPREAD);
        return target.direction.rotate(Vec2::from_angle(spread));
    }
    // Move to the side opposite the current heading, so a following orbit reverses direction.
    let heading_side = target.direction.perp().dot(heading);
    let tangent_sign = if heading_side.abs() > f32::EPSILON {
        -heading_side.signum()
    } else if rand::rng().random_bool(0.5) {
        1.0
    } else {
        -1.0
    };
    let tangent = target.direction.perp() * tangent_sign;
    (tangent - target.direction * HOP_OUTWARD_BIAS).normalize_or_zero()
}

/// Advances each dash in progress. Ends it after its distance or when its target is lost.
pub(super) fn apply_dash(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    targets: SteeringTargets,
    dashes: Query<&Dash>,
    mut dashers: Query<(
        Entity,
        SteeredCharacter,
        &RunningMotion,
        &mut DashProgress,
        &mut Steering,
    )>,
) {
    for (entity, character, running, mut progress, mut steering) in &mut dashers {
        let dash = running.source.and_then(|source| dashes.get(source).ok());
        let input = character.build_steer_input(running, &targets, &time);
        let step = match (dash, input) {
            (Some(dash), Some(input)) => progress.step(dash, &input),
            _ => MotionStep::Finished,
        };

        match step {
            MotionStep::Steer(next) => *steering = next,
            MotionStep::Finished => {
                commands
                    .entity(entity)
                    .try_remove::<(DashProgress, RunningMotion)>();
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum DashAimRule {
    Away { spread: f32 },
    HopTangent { close_range: f32 },
}

impl DashAimRule {
    fn choose_direction(self, target: &SteeringTarget, heading: Vec2) -> Vec2 {
        let direction = match self {
            Self::Away { spread } => (-target.direction)
                .normalize_or_zero()
                .rotate(Vec2::from_angle(rand::rng().random_range(-spread..=spread))),
            Self::HopTangent { close_range } => hop_direction(target, close_range, heading),
        };
        // Overlapping characters have no direction between them, so pick a random one.
        if direction.length_squared() > f32::EPSILON {
            direction
        } else {
            Vec2::from_angle(rand::rng().random_range(0.0..TAU))
        }
    }
}

/// A [`Dash`] in progress. Removing it, however the dash ended, starts its source's
/// [`MotionCooldown`](super::MotionCooldown) and fires [`MotionFinished`].
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(on_remove = end_dash)]
pub(super) struct DashProgress {
    direction: Option<Vec2>,
    remaining_distance: f32,
    blocked_timer: BlockedTimer,
}

impl DashProgress {
    fn choose_direction_and_distance(&mut self, dash: &Dash, input: &SteerInput) -> Option<Vec2> {
        let target = input.target.as_ref()?;
        let direction = dash.aim_rule.choose_direction(target, input.heading);
        self.direction = Some(direction);
        self.remaining_distance = dash.run_distance.pick_random_value().max(0.0);
        Some(direction)
    }

    fn step(&mut self, dash: &Dash, input: &SteerInput) -> MotionStep {
        let Some(direction) = self
            .direction
            .or_else(|| self.choose_direction_and_distance(dash, input))
        else {
            return MotionStep::Steer(Steering::boosted(None, dash.speed_multiplier));
        };

        if input.tick_blocked_timer(&mut self.blocked_timer, DASH_BLOCKED_SECONDS) {
            return MotionStep::Finished;
        }

        let distance = input.distance_this_frame(dash.speed_multiplier);
        if distance >= self.remaining_distance {
            return MotionStep::Finished;
        }

        self.remaining_distance -= distance;
        MotionStep::Steer(Steering::boosted(Some(direction), dash.speed_multiplier))
    }
}

fn end_dash(mut world: DeferredWorld, context: HookContext) {
    start_motion_source_cooldown(&mut world, context.entity);
    world.commands().trigger(MotionFinished {
        entity: context.entity,
        motion: MotionKind::Dash,
    });
}
