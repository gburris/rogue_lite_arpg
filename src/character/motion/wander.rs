use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::RngExt;

use super::{
    BlockedTimer, MotionKind, RunningMotion, SteerInput, SteeredCharacter, Steering,
    SteeringTargets,
};
use crate::interval::Interval;

const WANDER_SPEED_SCALE: f32 = 0.4;
/// Seconds a walk may stay blocked before it ends early.
const WANDER_BLOCKED_SECONDS: f32 = 0.3;

/// Alternates pausing and walking in a random direction. A blocked walk ends early.
#[derive(Component, Clone, Copy, Debug)]
#[require(MotionKind::Wander)]
pub struct Wander {
    pause: Interval,
    walk: Interval,
    speed_scale: f32,
}

/// A [`Wander`] in progress: its current phase and walk direction.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct WanderProgress {
    direction: Vec2,
    phase: Option<WanderPhase>,
    blocked_timer: BlockedTimer,
}

#[derive(Clone, Copy, Debug)]
enum WanderPhase {
    Walking { until: f32 },
    Pausing { until: f32 },
}

impl WanderPhase {
    fn until(self) -> f32 {
        match self {
            Self::Walking { until } | Self::Pausing { until } => until,
        }
    }
}

impl Wander {
    pub fn new(pause: Interval, walk: Interval) -> Self {
        Self {
            pause,
            walk,
            speed_scale: WANDER_SPEED_SCALE,
        }
    }
}

impl WanderProgress {
    /// Whether the character is walking rather than pausing.
    pub fn is_walking(&self) -> bool {
        matches!(self.phase, Some(WanderPhase::Walking { .. }))
    }

    fn pick_steering_direction(&mut self, wander: &Wander, input: &SteerInput) -> Option<Vec2> {
        let walk_blocked = self.is_walking()
            && input.tick_blocked_timer(&mut self.blocked_timer, WANDER_BLOCKED_SECONDS);

        let phase = match self.phase {
            Some(phase) if input.now < phase.until() && !walk_blocked => phase,
            previous => {
                let phase = if matches!(previous, Some(WanderPhase::Pausing { .. })) {
                    self.direction = pick_direction(input);
                    WanderPhase::Walking {
                        until: input.now + wander.walk.pick_random_value(),
                    }
                } else {
                    WanderPhase::Pausing {
                        until: input.now + wander.pause.pick_random_value(),
                    }
                };
                self.phase = Some(phase);
                phase
            }
        };
        match phase {
            WanderPhase::Walking { .. } => Some(self.direction),
            WanderPhase::Pausing { .. } => None,
        }
    }
}

/// Steers each character running [`Motion::Wander`](super::Motion::Wander).
pub(super) fn apply_wander(
    time: Res<Time<Virtual>>,
    targets: SteeringTargets,
    wanders: Query<&Wander>,
    mut wanderers: Query<(
        SteeredCharacter,
        &RunningMotion,
        &mut WanderProgress,
        &mut Steering,
    )>,
) {
    for (character, running, mut progress, mut steering) in &mut wanderers {
        let Some(wander) = running.source.and_then(|source| wanders.get(source).ok()) else {
            *steering = Steering::HALT;
            continue;
        };
        *steering = character
            .build_steer_input(running, &targets, &time)
            .map_or(Steering::HALT, |input| {
                let direction = progress.pick_steering_direction(wander, &input);
                Steering::scaled(direction, wander.speed_scale)
            });
    }
}

fn pick_direction(input: &SteerInput) -> Vec2 {
    let candidate = Vec2::from_angle(rand::rng().random_range(0.0..TAU));
    let Some(home_area) = input.home_area else {
        return candidate;
    };
    if home_area
        .overshoot_fraction(input.position)
        .is_none_or(|fraction| fraction < -0.25)
    {
        return candidate;
    }
    let Some(direction_home) = home_area.direction_home(input.position) else {
        return candidate;
    };
    direction_home.rotate(Vec2::from_angle(rand::rng().random_range(-0.6..0.6)))
}
