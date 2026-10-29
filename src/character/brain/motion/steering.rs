//! Per-frame steering: the [`Steering`] request each motion system writes, and
//! [`apply_steering`], which turns it into [`Kinematics`].

use std::time::Duration;

use bevy::prelude::*;

use crate::character::state::Kinematics;

/// Caps how fast a character turns, in radians per second. Absent means instant turns.
#[derive(Component, Clone, Copy, Debug)]
pub struct TurnRate(pub f32);

/// Extra speed left over from a finite motion's burst, easing off over
/// `duration`. Multiplies scaled steering until it decays.
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

    /// Advances the boost by `delta` and returns its speed multiplier, or
    /// `None` once it has decayed.
    fn tick_and_compute_speed_multiplier(&mut self, delta: Duration) -> Option<f32> {
        self.timer.tick(delta);
        if self.timer.is_finished() {
            return None;
        }
        Some(1.0 + self.extra_speed * self.easing.sample_clamped(self.timer.fraction_remaining()))
    }
}

/// This frame's steering request, written by the system that owns the held
/// motion.
#[derive(Component, Clone, Copy, Debug)]
pub(in crate::character::brain) struct Steering {
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

    /// Moves at a fixed `speed_multiplier` (at least `1.0`) times max speed. A finite
    /// motion measures its own distance at that speed, so leftover boosts stay out.
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

/// Turns each brain character's [`Steering`] into [`Kinematics`], capping how
/// fast it turns and dropping a [`DecayingSpeedBoost`] once it decays.
pub(super) fn apply_steering(
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    mut characters: Query<(
        Entity,
        &Steering,
        &mut Kinematics,
        Option<&TurnRate>,
        Option<&mut DecayingSpeedBoost>,
    )>,
) {
    let delta_secs = time.delta_secs();
    for (entity, steering, mut kinematics, turn_rate, mut decaying_speed_boost) in &mut characters {
        let boost_multiplier = decaying_speed_boost
            .as_mut()
            .and_then(|boost| boost.tick_and_compute_speed_multiplier(time.delta()));
        if decaying_speed_boost.is_some() && boost_multiplier.is_none() {
            commands.entity(entity).try_remove::<DecayingSpeedBoost>();
        }
        let Some(desired) = steering.direction else {
            if !kinematics.is_idle() {
                kinematics.stop_moving();
            }
            continue;
        };
        let speed_multiplier = match steering.effort {
            SteeringEffort::Scaled(scale) => {
                scale.clamp(0.0, 1.0) * boost_multiplier.unwrap_or(1.0)
            }
            SteeringEffort::Boosted(speed_multiplier) => speed_multiplier.max(1.0),
        };
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
