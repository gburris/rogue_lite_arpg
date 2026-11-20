use bevy::prelude::*;

use super::{
    BlockedTimer, MotionOf, MotionProgress, RunningMotion, SteerInput, SteeredCharacter, Steering,
    SteeringTargets, TunedBy,
};
use crate::interval::{Interval, Spread};

const WANDER_SPEED_SCALE: f32 = 0.4;
/// How far, in radians, a walk back toward home may stray from the straight line.
const WANDER_HOME_SPREAD: Spread = Spread::new(0.6);
/// Seconds a walk may stay blocked before it ends early.
const WANDER_BLOCKED_SECONDS: f32 = 0.3;

/// Alternates pausing and walking in a random direction. A blocked walk ends early.
#[derive(Component, Clone, Copy, Debug)]
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

impl MotionProgress for WanderProgress {
    type Tuning = Wander;
}

/// Steers each running motion tuned by a [`Wander`].
pub(super) fn apply_wander(
    time: Res<Time<Virtual>>,
    targets: SteeringTargets,
    characters: Query<SteeredCharacter>,
    wanders: Query<&Wander>,
    mut running_wanders: Query<(
        &MotionOf,
        &RunningMotion,
        &TunedBy,
        &mut WanderProgress,
        &mut Steering,
    )>,
) {
    for (motion_of, running, tuned_by, mut progress, mut steering) in &mut running_wanders {
        let wander = wanders.get(tuned_by.source).ok();
        let character = characters.get(motion_of.character).ok();
        let input = character
            .as_ref()
            .map(|character| character.build_steer_input(running.target, &targets, &time));
        *steering = match (wander, input) {
            (Some(wander), Some(input)) => {
                let direction = progress.pick_steering_direction(wander, &input);
                Steering::scaled(direction, wander.speed_scale)
            }
            _ => Steering::HALT,
        };
    }
}

fn pick_direction(input: &SteerInput) -> Vec2 {
    let candidate = Dir2::from_rng(&mut rand::rng()).as_vec2();
    let Some(home) = input.home else {
        return candidate;
    };
    if home
        .area
        .overshoot_fraction(home.center.distance)
        .is_none_or(|fraction| fraction < -0.25)
    {
        return candidate;
    }
    let Some(direction_home) = home.center.nonzero_direction() else {
        return candidate;
    };
    Rot2::radians(WANDER_HOME_SPREAD.pick_random_value()) * direction_home
}
