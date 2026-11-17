//! A character's [`Motion`] and the systems that steer it. Steering lives in [`steering`].

mod dash;
mod direct;
mod home_area;
mod orbit;
mod running_motion;
mod steering;
mod step;
mod swoop;
mod wander;

use bevy::prelude::*;

pub use self::dash::Dash;
use self::dash::DashProgress;
pub use self::home_area::HomeArea;
pub use self::orbit::Orbit;
use self::orbit::OrbitProgress;
use self::running_motion::RunningMotion;
pub(super) use self::running_motion::{MotionCommands, MotionFinished, MotionStatus, TargetRef};
use self::steering::DecayingSpeedBoost;
pub(super) use self::steering::Steering;
pub use self::steering::TurnRate;
use self::step::{
    BlockedTimer, MotionStep, SteerInput, SteeredCharacter, SteeringTarget, SteeringTargets,
};
pub use self::swoop::Swoop;
use self::swoop::SwoopProgress;
pub use self::wander::{Wander, WanderProgress};
use super::engine::Interval;
use crate::prelude::*;

/// How a character moves: a continuous motion that runs until replaced, or a
/// finite committed move that steers alone until it ends. Add a variant only for a new steering
/// contract.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Motion {
    /// Disengaged: alternates standing and ambling, paced by `Wander`.
    #[default]
    Wander,
    Chase,
    ReturnToHome,
    /// Wounded break: still in the fight, just getting away from the target.
    Flee,
    KeepDistance(Interval),
    /// Stands still on purpose. Scored like any motion, so it can win the
    /// slot and keep momentum; an empty slot also stands still, but nothing
    /// chose it.
    Idle,
    /// Circles the target inside the distance range.
    Orbit(Interval),
    /// Committed straight run; its `Dash` constructor (kite, hop, slide) sets its aim.
    Dash,
    /// Committed swoop through a snapshot of the target and back out.
    Swoop,
}

impl Motion {
    /// Every motion in array order. A payload here only stands in; arrays
    /// index by variant.
    pub(super) const ALL: &'static [Self] = &[
        Self::Wander,
        Self::Chase,
        Self::ReturnToHome,
        Self::Flee,
        Self::KeepDistance(Interval::new(0.0, 1.0)),
        Self::Idle,
        Self::Orbit(Interval::new(0.0, 1.0)),
        Self::Dash,
        Self::Swoop,
    ];

    /// Number of distinct motions, for per-motion arrays.
    pub(super) const COUNT: usize = Self::ALL.len();

    /// Position of this motion in per-motion arrays.
    pub(super) const fn index(self) -> usize {
        match self {
            Self::Wander => 0,
            Self::Chase => 1,
            Self::ReturnToHome => 2,
            Self::Flee => 3,
            Self::KeepDistance(_) => 4,
            Self::Idle => 5,
            Self::Orbit(_) => 6,
            Self::Dash => 7,
            Self::Swoop => 8,
        }
    }

    /// Whether both are the same variant, whatever their payloads.
    pub(super) fn is_same_kind(self, other: Self) -> bool {
        self.index() == other.index()
    }

    /// Whether this motion ends on its own, rather than running until replaced.
    pub(super) const fn is_finite(self) -> bool {
        match self {
            Self::Dash | Self::Swoop => true,
            Self::Wander
            | Self::Chase
            | Self::ReturnToHome
            | Self::Flee
            | Self::KeepDistance(_)
            | Self::Idle
            | Self::Orbit(_) => false,
        }
    }

    /// Whether steering this motion needs a live target.
    pub(super) fn needs_target(self) -> bool {
        match self {
            // Flee is still fighting: the target is live, the goal is just distance.
            Self::Chase
            | Self::KeepDistance(_)
            | Self::Orbit(_)
            | Self::Flee
            | Self::Dash
            | Self::Swoop => true,
            Self::Wander | Self::ReturnToHome | Self::Idle => false,
        }
    }

    /// One-word label for debug displays.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wander => "Wander",
            Self::Chase => "Chase",
            Self::ReturnToHome => "ReturnToHome",
            Self::Flee => "Flee",
            Self::KeepDistance(_) => "KeepDistance",
            Self::Idle => "Idle",
            Self::Orbit(_) => "Orbit",
            Self::Dash => "Dash",
            Self::Swoop => "Swoop",
        }
    }
}

// `ALL` lists every motion in `index` order.
const _: () = {
    // A new variant fails to compile here: add it to this match and to `ALL`.
    match Motion::Wander {
        Motion::Wander
        | Motion::Chase
        | Motion::ReturnToHome
        | Motion::Flee
        | Motion::KeepDistance(_)
        | Motion::Idle
        | Motion::Orbit(_)
        | Motion::Dash
        | Motion::Swoop => {}
    }
    let mut position = 0;
    while position < Motion::COUNT {
        assert!(
            Motion::ALL[position].index() == position,
            "Motion::ALL is out of index order"
        );
        position += 1;
    }
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            running_motion::end_interrupted_motions,
            dash::apply_dash,
            swoop::apply_swoop,
            orbit::apply_orbit,
            wander::apply_wander,
            home_area::settle_blocked_returners,
            direct::apply_direct_steering,
            steering::apply_steering,
        )
            .chain()
            .in_set(SimulationSystems::Steer),
    );
}
