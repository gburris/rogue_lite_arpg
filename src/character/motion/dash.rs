use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use bevy::prelude::*;

use super::{
    BlockedTimer, CircleDirection, FiniteMotion, MotionKey, MotionOf, MotionProgress, MotionStep,
    RunningMotion, SteerInput, SteeredCharacter, Steering, SteeringTarget, SteeringTargets,
    TunedBy,
};
use crate::interval::{Interval, Spread};

const KITE_SPREAD: Spread = Spread::new(FRAC_PI_4);

const STEP_IN_SPREAD: Spread = Spread::new(0.35);
/// Radians past the tangent a sidestep leans away from the target.
const SIDESTEP_OUTWARD_LEAN: f32 = 0.15;

/// Seconds a dash may stay blocked before it ends early.
const DASH_BLOCKED_SECONDS: f32 = 0.15;

/// A committed straight-line move in a direction chosen when it starts: `bearing_offset` radians
/// off the bearing to the target, turned against the way the character circles it, scattered by
/// `spread`. It ends early when blocked or interrupted. [`kite`], [`slide`], [`sidestep`] and
/// [`step_in`] build one with its [`MotionKey`].
#[derive(Component, Clone, Copy, Debug)]
#[require(FiniteMotion)]
pub struct Dash {
    run_distance: Interval,
    speed_multiplier: f32,
    bearing_offset: f32,
    spread: Spread,
}

impl Dash {
    fn new(
        run_distance: Interval,
        speed_multiplier: f32,
        bearing_offset: f32,
        spread: Spread,
    ) -> Self {
        Self {
            run_distance,
            speed_multiplier,
            bearing_offset,
            spread,
        }
    }

    fn choose_direction(&self, target: &SteeringTarget, heading: Vec2) -> Vec2 {
        let Some(bearing) = target.direction.try_normalize() else {
            // Overlapping characters have no direction between them, so pick a random one.
            return Dir2::from_rng(&mut rand::rng()).as_vec2();
        };
        let turn = CircleDirection::from_heading(bearing, heading).reversed();
        let aim = turn.find_direction_off_bearing(bearing, self.bearing_offset);
        Rot2::radians(self.spread.pick_random_value()) * aim
    }
}

/// A kite: runs away from the target at normal speed, scattered widely.
pub fn kite(run_distance: Interval) -> impl Scene {
    let dash = Dash::new(run_distance, 1.0, PI, KITE_SPREAD);
    bsn! {
        ~MotionKey::Kite
        ~{dash}
    }
}

/// A slide: runs straight away from the target.
pub fn slide(run_distance: Interval, speed_multiplier: f32) -> impl Scene {
    let dash = Dash::new(run_distance, speed_multiplier, PI, Spread::default());
    bsn! {
        ~MotionKey::Slide
        ~{dash}
    }
}

/// A sidestep: steps across the target's bearing, reversing the way the character circles it.
pub fn sidestep(run_distance: Interval, speed_multiplier: f32) -> impl Scene {
    let bearing_offset = FRAC_PI_2 + SIDESTEP_OUTWARD_LEAN;
    let dash = Dash::new(
        run_distance,
        speed_multiplier,
        bearing_offset,
        Spread::default(),
    );
    bsn! {
        ~MotionKey::Sidestep
        ~{dash}
    }
}

/// A step in: steps toward the target, scattered slightly.
pub fn step_in(run_distance: Interval, speed_multiplier: f32) -> impl Scene {
    let dash = Dash::new(run_distance, speed_multiplier, 0.0, STEP_IN_SPREAD);
    bsn! {
        ~MotionKey::StepIn
        ~{dash}
    }
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
        let direction = dash.choose_direction(target, heading);
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
