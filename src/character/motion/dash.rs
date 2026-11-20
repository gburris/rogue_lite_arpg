use bevy::prelude::*;

use super::{
    BlockedTimer, CircleDirection, FiniteMotion, MotionOf, MotionProgress, MotionStep,
    RunningMotion, SteerInput, SteeredCharacter, Steering, SteeringTarget, SteeringTargets,
    TunedBy,
};
use crate::interval::{Interval, Spread};

/// Speed below which a target counts as stationary.
const TARGET_MOVING_SPEED: f32 = 20.0;

const KITE_SPREAD: Spread = Spread::new(std::f32::consts::FRAC_PI_4);

const HOP_STEP_IN_SPREAD: Spread = Spread::new(0.35);
const HOP_OUTWARD_BIAS: f32 = 0.15;

/// Seconds a dash may stay blocked before it ends early.
const DASH_BLOCKED_SECONDS: f32 = 0.15;

/// A committed straight-line move in a direction chosen when it starts. It ends early when blocked
/// or interrupted.
#[derive(Component, Clone, Copy, Debug)]
#[require(FiniteMotion)]
pub struct Dash {
    run_distance: Interval,
    speed_multiplier: f32,
    aim_rule: DashAimRule,
}

impl Dash {
    fn new(run_distance: Interval, speed_multiplier: f32, aim_rule: DashAimRule) -> Self {
        Self {
            run_distance,
            speed_multiplier,
            aim_rule,
        }
    }

    pub fn kite(run_distance: Interval) -> Self {
        Self::new(
            run_distance,
            1.0,
            DashAimRule::Away {
                spread: KITE_SPREAD,
            },
        )
    }

    pub fn hop(run_distance: Interval, speed_multiplier: f32, close_range: f32) -> Self {
        Self::new(
            run_distance,
            speed_multiplier,
            DashAimRule::HopTangent { close_range },
        )
    }

    pub fn slide(run_distance: Interval, speed_multiplier: f32) -> Self {
        Self::new(
            run_distance,
            speed_multiplier,
            DashAimRule::Away {
                spread: Spread::default(),
            },
        )
    }
}

fn hop_direction(target: &SteeringTarget, close_range: f32, heading: Vec2) -> Option<Vec2> {
    if target.distance <= close_range
        && target.velocity.length_squared() <= TARGET_MOVING_SPEED * TARGET_MOVING_SPEED
    {
        let step_in = Rot2::radians(HOP_STEP_IN_SPREAD.pick_random_value()) * target.direction;
        return step_in.try_normalize();
    }
    // Move to the side opposite the current heading, so a following orbit reverses direction.
    let tangent = CircleDirection::from_heading(target.direction, heading)
        .reversed()
        .find_tangent(target.direction);
    (tangent - target.direction * HOP_OUTWARD_BIAS).try_normalize()
}

impl MotionProgress for DashProgress {
    type Tuning = Dash;
}

/// Advances each dash in progress. Ends it after its distance or when its target is lost.
pub(super) fn apply_dash(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    targets: SteeringTargets,
    characters: Query<SteeredCharacter>,
    dashes: Query<&Dash>,
    mut running_dashes: Query<(
        Entity,
        &MotionOf,
        &RunningMotion,
        &TunedBy,
        &mut DashProgress,
        &mut Steering,
    )>,
) {
    for (entity, motion_of, running, tuned_by, mut progress, mut steering) in &mut running_dashes {
        let dash = dashes.get(tuned_by.source).ok();
        let character = characters.get(motion_of.character).ok();
        let input = character
            .as_ref()
            .map(|character| character.build_steer_input(running.target, &targets, &time));
        let step = match (dash, input) {
            (
                Some(dash),
                Some(
                    input @ SteerInput {
                        target: Some(target),
                        ..
                    },
                ),
            ) => progress.step(dash, &input, &target),
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

#[derive(Clone, Copy, Debug)]
enum DashAimRule {
    Away { spread: Spread },
    HopTangent { close_range: f32 },
}

impl DashAimRule {
    fn choose_direction(self, target: &SteeringTarget, heading: Vec2) -> Vec2 {
        let direction = match self {
            Self::Away { spread } => (-target.direction)
                .try_normalize()
                .map(|away| Rot2::radians(spread.pick_random_value()) * away),
            Self::HopTangent { close_range } => hop_direction(target, close_range, heading),
        };
        // Overlapping characters have no direction between them, so pick a random one.
        direction.unwrap_or_else(|| Dir2::from_rng(&mut rand::rng()).as_vec2())
    }
}

/// A [`Dash`] in progress, on its running motion.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct DashProgress {
    direction: Option<Vec2>,
    remaining_distance: f32,
    blocked_timer: BlockedTimer,
}

impl DashProgress {
    fn choose_direction_and_distance(
        &mut self,
        dash: &Dash,
        target: &SteeringTarget,
        heading: Vec2,
    ) -> Vec2 {
        let direction = dash.aim_rule.choose_direction(target, heading);
        self.direction = Some(direction);
        self.remaining_distance = dash.run_distance.pick_random_value().max(0.0);
        direction
    }

    fn step(&mut self, dash: &Dash, input: &SteerInput, target: &SteeringTarget) -> MotionStep {
        let direction = match self.direction {
            Some(direction) => direction,
            None => self.choose_direction_and_distance(dash, target, input.heading),
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
