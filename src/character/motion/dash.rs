use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use bevy::prelude::*;

use super::{
    BlockedTimer, CircleDirection, FiniteMotion, MotionKey, MotionProgress, MotionStep, SteerInput,
    Steering, SteeringTarget,
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
/// [`step_in`] build its motion sources, each with its own [`MotionKey`].
#[derive(Component, Clone, Copy, Debug)]
#[require(FiniteMotion)]
pub(super) struct Dash {
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
        let Some(bearing) = target.nonzero_direction() else {
            // On top of the target there is no bearing, so pick a random direction.
            return Dir2::from_rng(&mut rand::rng()).as_vec2();
        };
        let turn = CircleDirection::from_heading(bearing, heading).reversed();
        let aim = turn.find_direction_off_bearing(bearing, self.bearing_offset);
        Rot2::radians(self.spread.pick_random_value()) * aim
    }
}

/// Dashes away from the target at normal speed, scattered widely.
pub fn kite(run_distance: Interval) -> impl Scene {
    let dash = Dash::new(run_distance, 1.0, PI, KITE_SPREAD);
    bsn! {
        ~MotionKey::Kite
        ~{dash}
    }
}

/// Dashes straight away from the target.
pub fn slide(run_distance: Interval, speed_multiplier: f32) -> impl Scene {
    let dash = Dash::new(run_distance, speed_multiplier, PI, Spread::default());
    bsn! {
        ~MotionKey::Slide
        ~{dash}
    }
}

/// Dashes across the bearing to the target, reversing the way the character circles it.
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

/// Dashes toward the target, scattered slightly.
pub fn step_in(run_distance: Interval, speed_multiplier: f32) -> impl Scene {
    let dash = Dash::new(run_distance, speed_multiplier, 0.0, STEP_IN_SPREAD);
    bsn! {
        ~MotionKey::StepIn
        ~{dash}
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
}

impl MotionProgress for DashProgress {
    type Tuning = Dash;

    /// Ends after the dash's distance, or when blocked or the target is lost.
    fn step(&mut self, dash: &Dash, input: &SteerInput) -> MotionStep {
        let Some(target) = input.target else {
            return MotionStep::Finished;
        };
        let direction = match self.direction {
            Some(direction) => direction,
            None => self.choose_direction_and_distance(dash, &target, input.heading),
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
