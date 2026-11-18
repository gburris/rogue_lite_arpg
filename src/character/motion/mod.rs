mod dash;
mod direct;
mod home_area;
mod orbit;
mod running_motion;
mod steer_input;
mod steering;
mod swoop;
mod wander;

use bevy::prelude::*;

pub use self::dash::Dash;
use self::dash::DashProgress;
pub use self::home_area::HomeArea;
pub use self::orbit::Orbit;
use self::orbit::OrbitProgress;
use self::running_motion::RunningMotion;
pub(super) use self::running_motion::{MotionCommands, MotionFinished, MotionStatus};
pub(super) use self::steer_input::AssumedVelocity;
use self::steer_input::{
    BlockedTimer, MotionStep, SteerInput, SteeredCharacter, SteeringTarget, SteeringTargets,
};
pub use self::steering::TurnRate;
use self::steering::{DecayingSpeedBoost, Steering};
pub use self::swoop::Swoop;
use self::swoop::SwoopProgress;
pub use self::wander::{Wander, WanderProgress};
use crate::interval::Interval;
use crate::prelude::*;

/// How a character moves: a continuous motion that runs until replaced, or a finite motion that
/// ends on its own. Add a variant only for a new kind of steering.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Motion {
    /// Alternates pausing and walking; see [`Wander`].
    #[default]
    Wander,
    Chase,
    ReturnToHome,
    /// Moves directly away from the target.
    Flee,
    KeepDistance(Interval),
    /// Stands still. Unlike an empty motion slot, it is scored and chosen like any other motion.
    Idle,
    /// Circles the target within the distance range; see [`Orbit`].
    Orbit(Interval),
    /// See [`Dash`].
    Dash,
    /// See [`Swoop`].
    Swoop,
}

impl Motion {
    /// Every motion in [`Self::index`] order. Payloads are placeholders.
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

    /// Whether steering this motion needs a target.
    pub(super) fn needs_target(self) -> bool {
        match self {
            // Flee steers away from the target, so it needs one.
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
            home_area::move_homes_of_blocked_characters,
            direct::apply_direct_steering,
            steering::decay_speed_boosts,
            steering::apply_steering,
        )
            .chain()
            .in_set(SimulationSystems::Steer),
    );
}
