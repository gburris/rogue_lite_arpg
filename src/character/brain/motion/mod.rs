//! A character's [`Motion`] and the systems that steer it. Steering lives in [`steering`].

mod dash;
mod direct;
mod home_area;
mod orbit;
mod steering;
mod step;
mod swoop;
mod wander;

use bevy::prelude::*;

pub use self::dash::Dash;
pub use self::home_area::HomeArea;
pub use self::orbit::Orbit;
use self::steering::DecayingSpeedBoost;
pub(super) use self::steering::Steering;
pub use self::steering::TurnRate;
pub use self::step::MotionFinished;
use self::step::{
    CurrentBehaviorRunId, MotionStep, SteerInput, SteeredCharacter, report_motion_finished,
};
pub use self::swoop::Swoop;
pub use self::wander::Wander;
use super::BrainSystems;
use super::engine::Interval;

/// How a character moves: a continuous motion that runs until replaced, or a
/// finite committed move that steers alone until its system reports it
/// finished. Add a variant only for a new steering contract.
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
    /// Committed dive through a snapshot of the target and back out.
    Swoop,
}

impl Motion {
    /// Every motion in array order. A payload here only stands in; arrays
    /// index by variant.
    pub const ALL: &'static [Self] = &[
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
    pub const COUNT: usize = Self::ALL.len();

    /// Position of this motion in per-motion arrays.
    pub const fn index(self) -> usize {
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

    /// Whether this motion ends on its own and reports it, rather than
    /// running until replaced.
    pub const fn is_finite(self) -> bool {
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
    pub fn needs_target(self) -> bool {
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
            dash::apply_dash,
            swoop::apply_swoop,
            orbit::apply_orbit,
            wander::apply_wander,
            direct::apply_direct_steering,
            steering::apply_steering,
        )
            .chain()
            .in_set(BrainSystems::Motion),
    );
}
