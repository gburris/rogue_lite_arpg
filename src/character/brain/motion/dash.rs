//! Committed straight runs: the warrior's kite, the fire mage's hop and the ice mage's slide.

use bevy::prelude::*;

use super::{
    CurrentBehaviorRunId, MotionStep, SteerInput, SteeredCharacter, Steering,
    report_motion_finished,
};
use crate::character::brain::perception::{TargetView, build_target_view};
use crate::character::brain::{BrainRng, Capability, CapabilityProfile, Interval, Motion};

/// Speed below which a target counts as holding still rather than orbiting.
const TARGET_MOVING_SPEED: f32 = 20.0;

const KITE_SPREAD: f32 = std::f32::consts::FRAC_PI_4;

const HOP_STEP_IN_SPREAD: f32 = 0.35;
const HOP_OUTWARD_BIAS: f32 = 0.15;

/// A committed straight run along a direction fixed at its start.
#[derive(Component, Clone, Debug)]
pub struct Dash {
    pub run_distance: Interval,
    pub speed_multiplier: f32,
    pub capability_profile: CapabilityProfile,
    aim_rule: DashAimRule,
    debug_name: &'static str,
    progress: DashProgress,
    current_behavior_run_id: CurrentBehaviorRunId,
}

impl Dash {
    fn with_aim_rule(
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
            progress: DashProgress::default(),
            current_behavior_run_id: CurrentBehaviorRunId::default(),
        }
    }

    pub fn kite(chance: f32, run_distance: Interval) -> Self {
        Self::with_aim_rule(
            run_distance,
            1.0,
            CapabilityProfile {
                chance,
                ..CapabilityProfile::default()
            },
            DashAimRule::Away {
                spread: KITE_SPREAD,
            },
            "Kite",
        )
    }

    pub fn hop(run_distance: Interval, speed_multiplier: f32, close_range: f32) -> Self {
        Self::with_aim_rule(
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
        Self::with_aim_rule(
            run_distance,
            speed_multiplier,
            CapabilityProfile {
                reach: Some(danger_range),
                cooldown,
                ..CapabilityProfile::default()
            },
            DashAimRule::Away { spread: 0.0 },
            "Slide",
        )
    }

    pub fn name(&self) -> &'static str {
        self.debug_name
    }
}

fn hop_direction(target: &TargetView, close_range: f32, heading: Vec2, rng: &mut BrainRng) -> Vec2 {
    if target.distance <= close_range
        && target.velocity.length_squared() <= TARGET_MOVING_SPEED * TARGET_MOVING_SPEED
    {
        let spread = rng.range(-HOP_STEP_IN_SPREAD, HOP_STEP_IN_SPREAD);
        return target.bearing.rotate(Vec2::from_angle(spread));
    }
    // Cut across to the side opposite the current heading, so an orbit that
    // follows reads the new heading and circles back the other way.
    let heading_side = target.bearing.perp().dot(heading);
    let tangent_sign = if heading_side.abs() > f32::EPSILON {
        -heading_side.signum()
    } else if rng.range(0.0, 1.0) < 0.5 {
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

    fn capability_profile(&self) -> CapabilityProfile {
        self.capability_profile
    }
}

/// Runs a held dash and reports [`MotionFinished`](super::MotionFinished) at
/// the end of its distance.
pub(super) fn apply_dash(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut dashers: Query<(
        Entity,
        SteeredCharacter,
        &mut Dash,
        &mut Steering,
        &mut BrainRng,
    )>,
) {
    for (entity, character, mut dash, mut steering, mut rng) in &mut dashers {
        if dash
            .current_behavior_run_id
            .follow_brain_run(character.brain, Motion::Dash)
        {
            dash.progress.reset();
        }
        let Some(behavior_run_id) = dash.current_behavior_run_id.get() else {
            continue;
        };
        let Some(input) = character.build_steer_input(Motion::Dash, &time) else {
            *steering = Steering::HALT;
            continue;
        };
        match dash.step(&input, &mut rng) {
            MotionStep::Steer(next) => *steering = next,
            MotionStep::Finished {
                decaying_speed_boost,
            } => {
                report_motion_finished(
                    &mut commands,
                    entity,
                    Motion::Dash,
                    behavior_run_id,
                    decaying_speed_boost,
                );
            }
        }
    }
}

impl Dash {
    fn step(&mut self, input: &SteerInput, rng: &mut BrainRng) -> MotionStep {
        if !self.progress.started {
            // A target with no finite distance gives nothing to aim at, so this
            // waits rather than aiming blind.
            let Some(target) = build_target_view(true, input.target_info, input.now) else {
                return MotionStep::Steer(Steering::boosted(None, self.speed_multiplier));
            };
            let direction = self.aim_rule.choose_direction(target, input.heading, rng);
            let run = rng.range(self.run_distance.min(), self.run_distance.max());
            self.progress.start(direction, run);
        }
        let distance = input.distance_this_frame(self.speed_multiplier);
        match self.progress.step(distance) {
            Some(direction) => {
                MotionStep::Steer(Steering::boosted(Some(direction), self.speed_multiplier))
            }
            None => MotionStep::Finished {
                decaying_speed_boost: None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum DashAimRule {
    Away { spread: f32 },
    HopTangent { close_range: f32 },
}

impl DashAimRule {
    fn choose_direction(self, target: TargetView, heading: Vec2, rng: &mut BrainRng) -> Vec2 {
        let direction = match self {
            Self::Away { spread } => (-target.bearing)
                .normalize_or_zero()
                .rotate(Vec2::from_angle(rng.range(-spread, spread))),
            Self::HopTangent { close_range } => hop_direction(&target, close_range, heading, rng),
        };
        // A zero bearing (the characters overlap) has no direction to aim along.
        if direction.length_squared() > f32::EPSILON {
            direction
        } else {
            rng.direction()
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct DashProgress {
    direction: Vec2,
    remaining: f32,
    started: bool,
}

impl DashProgress {
    fn start(&mut self, direction: Vec2, run: f32) {
        self.direction = direction;
        self.remaining = run.max(0.0);
        self.started = true;
    }

    fn reset(&mut self) {
        self.started = false;
        self.remaining = 0.0;
    }

    fn step(&mut self, distance: f32) -> Option<Vec2> {
        if !self.started {
            return None;
        }
        if distance >= self.remaining {
            self.reset();
            return None;
        }
        self.remaining -= distance;
        Some(self.direction)
    }
}
