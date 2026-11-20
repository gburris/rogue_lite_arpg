use bevy::prelude::*;

use super::{
    MotionKey, MotionOf, RunningMotion, SteerInput, SteeredCharacter, Steering, SteeringTargets,
    TunedBy,
};
use crate::interval::Interval;

const KEEP_ARRIVAL_DEADBAND: f32 = 4.0;

/// A continuous motion that steers straight from its tuning and this frame's input, with no
/// progress of its own. [`apply_direct_steering`] steers it.
pub(super) trait DirectMotion: Component {
    /// The direction to move this frame; `None` stands still.
    fn pick_steering_direction(&self, input: &SteerInput) -> Option<Vec2>;
}

/// Stands still.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(MotionKey::Idle)]
pub struct Idle;

impl DirectMotion for Idle {
    fn pick_steering_direction(&self, _input: &SteerInput) -> Option<Vec2> {
        None
    }
}

/// Moves straight at the target.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(MotionKey::Chase)]
pub struct Chase;

impl DirectMotion for Chase {
    fn pick_steering_direction(&self, input: &SteerInput) -> Option<Vec2> {
        input.direction_to_target()
    }
}

/// Moves directly away from the target.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(MotionKey::Flee)]
pub struct Flee;

impl DirectMotion for Flee {
    fn pick_steering_direction(&self, input: &SteerInput) -> Option<Vec2> {
        input.direction_from_target()
    }
}

/// Holds the target within a distance range, backing away when too close and closing in when too
/// far.
#[derive(Component, Clone, Copy, Debug)]
#[require(MotionKey::KeepDistance)]
pub struct KeepDistance {
    distance_range: Interval,
}

impl KeepDistance {
    /// Holds the target within `distance_range`.
    pub fn new(distance_range: Interval) -> Self {
        Self { distance_range }
    }
}

impl DirectMotion for KeepDistance {
    fn pick_steering_direction(&self, input: &SteerInput) -> Option<Vec2> {
        let distance = input.target?.distance;
        let Some(direction) = input.direction_to_target() else {
            // On top of the target there is no direction to it, so keep the current heading.
            if distance < self.distance_range.min() {
                return Some(
                    input
                        .heading
                        .try_normalize()
                        .unwrap_or_else(|| Dir2::from_rng(&mut rand::rng()).as_vec2()),
                );
            }
            return None;
        };
        let error = self.distance_range.error(distance);
        if error.abs() < KEEP_ARRIVAL_DEADBAND {
            None
        } else if error < 0.0 {
            Some(-direction)
        } else {
            Some(direction)
        }
    }
}

/// Steers each running motion tuned by a `Tuning` source.
pub(super) fn apply_direct_steering<Tuning: DirectMotion>(
    time: Res<Time<Virtual>>,
    targets: SteeringTargets,
    characters: Query<SteeredCharacter>,
    tunings: Query<&Tuning>,
    mut running_motions: Query<(&MotionOf, &RunningMotion, &TunedBy, &mut Steering)>,
) {
    for (motion_of, running, tuned_by, mut steering) in &mut running_motions {
        let Ok(tuning) = tunings.get(tuned_by.source) else {
            continue;
        };
        *steering = characters
            .get(motion_of.character)
            .map_or(Steering::HALT, |character| {
                let input = character.build_steer_input(running.target, &targets, &time);
                Steering::scaled(tuning.pick_steering_direction(&input), 1.0)
            });
    }
}
