//! Committed straight runs: the warrior's kite, the fire mage's hop and the ice mage's slide.

use std::f32::consts::TAU;

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::RngExt;

use super::{
    BlockedTimer, MotionFinished, MotionStep, RunningMotion, SteerInput, SteeredCharacter,
    Steering, SteeringTarget, SteeringTargets,
};
use crate::character::brain::{CapabilityProfile, Interval, Motion};

/// Speed below which a target counts as holding still rather than orbiting.
const TARGET_MOVING_SPEED: f32 = 20.0;

const KITE_SPREAD: f32 = std::f32::consts::FRAC_PI_4;

const HOP_STEP_IN_SPREAD: f32 = 0.35;
const HOP_OUTWARD_BIAS: f32 = 0.15;

/// Seconds a dash pushes against a wall or a crowd before it gives up the rest of its run.
const DASH_BLOCKED_SECONDS: f32 = 0.15;

/// A committed straight run along a direction fixed at its start, cut short when blocked or
/// interrupted.
#[derive(Component, Clone, Copy, Debug)]
pub struct Dash {
    run_distance: Interval,
    speed_multiplier: f32,
    capability_profile: CapabilityProfile,
    aim_rule: DashAimRule,
    debug_name: &'static str,
}

impl Dash {
    fn new(
        run_distance: Interval,
        speed_multiplier: f32,
        capability_profile: CapabilityProfile,
        aim_rule: DashAimRule,
        debug_name: &'static str,
    ) -> Self {
        Self {
            run_distance,
            speed_multiplier,
            capability_profile,
            aim_rule,
            debug_name,
        }
    }

    pub fn kite(run_distance: Interval) -> Self {
        Self::new(
            run_distance,
            1.0,
            CapabilityProfile::default(),
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
            CapabilityProfile::default(),
            DashAimRule::HopTangent { close_range },
            "Hop",
        )
    }

    pub fn slide(
        run_distance: Interval,
        speed_multiplier: f32,
        danger_range: f32,
        cooldown: f32,
    ) -> Self {
        Self::new(
            run_distance,
            speed_multiplier,
            CapabilityProfile {
                reach: Some(danger_range),
                cooldown,
            },
            DashAimRule::Away { spread: 0.0 },
            "Slide",
        )
    }

    pub fn name(&self) -> &'static str {
        self.debug_name
    }

    /// What a decision reads of this capability.
    pub(in crate::character::brain) fn capability_profile(&self) -> CapabilityProfile {
        self.capability_profile
    }
}

fn hop_direction(target: &SteeringTarget, close_range: f32, heading: Vec2) -> Vec2 {
    if target.distance <= close_range
        && target.velocity.length_squared() <= TARGET_MOVING_SPEED * TARGET_MOVING_SPEED
    {
        let spread = rand::rng().random_range(-HOP_STEP_IN_SPREAD..=HOP_STEP_IN_SPREAD);
        return target.direction.rotate(Vec2::from_angle(spread));
    }
    // Cut across to the side opposite the current heading, so an orbit that
    // follows reads the new heading and circles back the other way.
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

/// Advances each running dash and ends it at the end of its distance or once its target is lost.
pub(super) fn apply_dash(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    targets: SteeringTargets,
    mut dashers: Query<(
        Entity,
        SteeredCharacter,
        &RunningMotion,
        &Dash,
        &mut DashProgress,
        &mut Steering,
    )>,
) {
    for (entity, character, running, dash, mut progress, mut steering) in &mut dashers {
        let step = match character.build_steer_input(running, &targets, &time) {
            Some(input) => progress.step(dash, &input),
            None => MotionStep::Finished,
        };
        match step {
            MotionStep::Steer(next) => *steering = next,
            MotionStep::Finished => {
                commands.entity(entity).try_remove::<DashProgress>();
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
        // A zero bearing (the characters overlap) has no direction to aim along.
        if direction.length_squared() > f32::EPSILON {
            direction
        } else {
            Vec2::from_angle(rand::rng().random_range(0.0..TAU))
        }
    }
}

/// One run of a character's [`Dash`], present from its start until it ends. Its removal, however
/// the run ended, reports [`MotionFinished`].
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(on_remove = report_dash_finished)]
pub(super) struct DashProgress {
    direction: Option<Vec2>,
    remaining: f32,
    blocked_timer: BlockedTimer,
}

impl DashProgress {
    fn aim_run(&mut self, dash: &Dash, input: &SteerInput) -> Option<Vec2> {
        let target = input.target.as_ref()?;
        let direction = dash.aim_rule.choose_direction(target, input.heading);
        self.direction = Some(direction);
        self.remaining = dash.run_distance.pick_random_value().max(0.0);
        Some(direction)
    }

    fn step(&mut self, dash: &Dash, input: &SteerInput) -> MotionStep {
        let Some(direction) = self.direction.or_else(|| self.aim_run(dash, input)) else {
            return MotionStep::Steer(Steering::boosted(None, dash.speed_multiplier));
        };
        if input.tick_blocked_timer(&mut self.blocked_timer, DASH_BLOCKED_SECONDS) {
            return MotionStep::Finished;
        }
        let distance = input.distance_this_frame(dash.speed_multiplier);
        if distance >= self.remaining {
            return MotionStep::Finished;
        }
        self.remaining -= distance;
        MotionStep::Steer(Steering::boosted(Some(direction), dash.speed_multiplier))
    }
}

fn report_dash_finished(mut world: DeferredWorld, context: HookContext) {
    world.commands().trigger(MotionFinished {
        entity: context.entity,
        motion: Motion::Dash,
    });
}
