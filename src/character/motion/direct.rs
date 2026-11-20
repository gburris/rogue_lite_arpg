use bevy::prelude::*;

use super::{
    Motion, MotionOf, RunningMotion, SteerInput, SteeredCharacter, Steering, SteeringTargets,
    TunedBy,
};
use crate::interval::Interval;

const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// Holds the target within a distance range, backing away when too close and closing in when too
/// far.
#[derive(Component, Clone, Copy, Debug)]
#[require(Motion::KeepDistance)]
pub struct KeepDistance {
    distance_range: Interval,
}

impl KeepDistance {
    /// Holds the target within `distance_range`.
    pub fn new(distance_range: Interval) -> Self {
        Self { distance_range }
    }
}

/// Steers each running motion without a system of its own.
pub(super) fn apply_direct_steering(
    time: Res<Time<Virtual>>,
    targets: SteeringTargets,
    characters: Query<SteeredCharacter>,
    keep_distances: Query<&KeepDistance>,
    mut running_motions: Query<(&MotionOf, &RunningMotion, Option<&TunedBy>, &mut Steering)>,
) {
    for (motion_of, running, tuned_by, mut steering) in &mut running_motions {
        let motion = match running.motion {
            // Steered by their own systems.
            Motion::Wander | Motion::Orbit | Motion::Dash | Motion::Swoop => continue,
            motion => motion,
        };
        let keep_distance = tuned_by.and_then(|tuned_by| keep_distances.get(tuned_by.source).ok());
        let character = characters.get(motion_of.character).ok();
        let input = character
            .as_ref()
            .and_then(|character| character.build_steer_input(running, &targets, &time));
        *steering = input.map_or(Steering::HALT, |input| {
            Steering::scaled(steer_direct_motion(motion, keep_distance, &input), 1.0)
        });
    }
}

fn steer_direct_motion(
    motion: Motion,
    keep_distance: Option<&KeepDistance>,
    input: &SteerInput,
) -> Option<Vec2> {
    match motion {
        Motion::Chase => input.direction_to_target(),
        Motion::Flee => input.direction_from_target(),
        Motion::ReturnToHome => input.direction_to_home(),
        Motion::KeepDistance => keep_distance.and_then(|keep_distance| {
            steer_to_hold_distance_range(input, keep_distance.distance_range)
        }),
        Motion::Idle | Motion::Wander | Motion::Orbit | Motion::Dash | Motion::Swoop => None,
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
                    .unwrap_or_else(|| Dir2::from_rng(&mut rand::rng()).as_vec2()),
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
