//! Motions with no state of their own: chase, flee, return home, hold a
//! distance, and standing still.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::RngExt;

use super::{Motion, SteerInput, SteeredCharacter, Steering};
use crate::character::brain::Interval;

const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// Steers every held motion no component owns, including an empty slot.
pub(super) fn apply_direct_steering(
    time: Res<Time<Virtual>>,
    mut characters: Query<(SteeredCharacter, &mut Steering)>,
) {
    for (character, mut steering) in &mut characters {
        *steering = match character.brain.motion() {
            // Steered by their components' systems.
            Some(Motion::Wander | Motion::Orbit(_) | Motion::Dash | Motion::Swoop) => continue,
            None => Steering::HALT,
            Some(motion) => character
                .build_steer_input(motion, &time)
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
    let distance = input.target_info.map_or(0.0, |info| info.distance);
    let Some(direction) = input.direction_to_target() else {
        // Sitting on the target leaves no bearing, so keep the current heading
        // to peel off it.
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
