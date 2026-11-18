use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::RngExt;

use super::{Motion, RunningMotion, SteerInput, SteeredCharacter, Steering, SteeringTargets};
use crate::interval::Interval;

const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// Steers each running motion without a system of its own.
pub(super) fn apply_direct_steering(
    time: Res<Time<Virtual>>,
    targets: SteeringTargets,
    mut characters: Query<(SteeredCharacter, &RunningMotion, &mut Steering)>,
) {
    for (character, running, mut steering) in &mut characters {
        *steering = match running.motion {
            // Steered by their own systems.
            Motion::Wander | Motion::Orbit(_) | Motion::Dash | Motion::Swoop => continue,
            motion => character
                .build_steer_input(running, &targets, &time)
                .map_or(Steering::HALT, |input| {
                    Steering::scaled(steer_direct_motion(motion, &input), 1.0)
                }),
        };
    }
}

fn steer_direct_motion(motion: Motion, input: &SteerInput) -> Option<Vec2> {
    match motion {
        Motion::Chase => input.direction_to_target(),
        Motion::Flee => input.direction_from_target(),
        Motion::ReturnToHome => input.direction_to_home(),
        Motion::KeepDistance(distance_range) => steer_to_hold_distance_range(input, distance_range),
        Motion::Idle | Motion::Wander | Motion::Orbit(_) | Motion::Dash | Motion::Swoop => None,
    }
}

fn steer_to_hold_distance_range(input: &SteerInput, distance_range: Interval) -> Option<Vec2> {
    let distance = input.target.map_or(0.0, |target| target.distance);
    let Some(direction) = input.direction_to_target() else {
        // On top of the target there is no direction to it, so keep the current heading.
        if distance < distance_range.min() {
            return Some(
                input
                    .heading
                    .try_normalize()
                    .unwrap_or_else(|| Vec2::from_angle(rand::rng().random_range(0.0..TAU))),
            );
        }
        return None;
    };
    let error = distance_range.error(distance);
    if error.abs() < KEEP_ARRIVAL_DEADBAND {
        None
    } else if error < 0.0 {
        Some(-direction)
    } else {
        Some(direction)
    }
}
