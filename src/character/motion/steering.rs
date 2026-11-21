use bevy::prelude::*;

use super::RunningMotionOf;
use crate::character::state::Kinematics;

/// Caps how fast a character turns, in radians per second. Absent means instant turns.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TurnRate(pub f32);

/// Extra speed left after a finite motion ends, decreasing to zero over `duration`. Multiplies
/// scaled steering until then.
#[derive(Component, Clone, Debug)]
pub(super) struct DecayingSpeedBoost {
    extra_speed: f32,
    timer: Timer,
    easing: EaseFunction,
}

impl DecayingSpeedBoost {
    pub(super) fn new(extra_speed: f32, duration: f32, easing: EaseFunction) -> Self {
        Self {
            extra_speed,
            timer: Timer::from_seconds(duration, TimerMode::Once),
            easing,
        }
    }

    fn speed_multiplier(&self) -> f32 {
        1.0 + self.extra_speed * self.easing.sample_clamped(self.timer.fraction_remaining())
    }
}

/// Advances each [`DecayingSpeedBoost`], removing it once it has decayed.
pub(super) fn decay_speed_boosts(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut characters: Query<(Entity, &mut DecayingSpeedBoost)>,
) {
    for (entity, mut boost) in &mut characters {
        boost.timer.tick(time.delta());
        if boost.timer.is_finished() {
            commands.entity(entity).try_remove::<DecayingSpeedBoost>();
        }
    }
}

/// This frame's steering request, on a running motion, written by that motion's system.
#[derive(Component, Clone, Copy, Debug)]
pub(in crate::character) struct Steering {
    direction: Option<Vec2>,
    effort: SteeringEffort,
}

impl Default for Steering {
    fn default() -> Self {
        Self::HALT
    }
}

impl Steering {
    pub(super) const HALT: Self = Self::scaled(None, 1.0);

    /// Moves at `scale` (clamped to `0..=1`) times max speed, raised by any
    /// leftover [`DecayingSpeedBoost`].
    pub(super) const fn scaled(direction: Option<Vec2>, scale: f32) -> Self {
        Self {
            direction,
            effort: SteeringEffort::Scaled(scale),
        }
    }

    /// Moves at a fixed `speed_multiplier` (at least `1.0`) times max speed, ignoring any
    /// [`DecayingSpeedBoost`], since a finite motion measures its distance at this speed.
    pub(super) const fn boosted(direction: Option<Vec2>, speed_multiplier: f32) -> Self {
        Self {
            direction,
            effort: SteeringEffort::Boosted(speed_multiplier),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum SteeringEffort {
    Scaled(f32),
    Boosted(f32),
}

impl SteeringEffort {
    fn speed_multiplier(self, boost_multiplier: Option<f32>) -> f32 {
        match self {
            Self::Scaled(scale) => scale.clamp(0.0, 1.0) * boost_multiplier.unwrap_or(1.0),
            Self::Boosted(speed_multiplier) => speed_multiplier.max(1.0),
        }
    }
}

/// Turns the [`Steering`] of each running motion into its character's [`Kinematics`], limiting
/// the turn rate.
pub(super) fn apply_steering(
    time: Res<Time<Virtual>>,
    running_motions: Query<(&RunningMotionOf, &Steering)>,
    mut characters: Query<(
        &mut Kinematics,
        Option<&TurnRate>,
        Option<&DecayingSpeedBoost>,
    )>,
) {
    let delta_secs = time.delta_secs();
    for (running_motion_of, steering) in &running_motions {
        let Ok((mut kinematics, turn_rate, decaying_speed_boost)) =
            characters.get_mut(running_motion_of.character)
        else {
            continue;
        };
        let boost_multiplier = decaying_speed_boost.map(DecayingSpeedBoost::speed_multiplier);
        let Some(desired) = steering.direction else {
            if !kinematics.is_idle() {
                kinematics.stop_moving();
            }
            continue;
        };
        let speed_multiplier = steering.effort.speed_multiplier(boost_multiplier);
        let turn_cap = turn_rate.map(|rate| rate.0 * delta_secs);
        let heading = rotate_heading_toward_desired(kinematics.direction, desired, turn_cap);
        kinematics.move_in_direction(heading, speed_multiplier);
    }
}

fn rotate_heading_toward_desired(heading: Vec2, desired: Vec2, turn_cap: Option<f32>) -> Vec2 {
    if let Some(max_angle) = turn_cap
        && heading.length_squared() > f32::EPSILON
        && desired.length_squared() > f32::EPSILON
    {
        heading.rotate_towards(desired, max_angle)
    } else {
        desired
    }
}
