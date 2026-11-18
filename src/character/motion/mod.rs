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
    /// Which motion this is, without its settings.
    pub const fn kind(self) -> MotionKind {
        match self {
            Self::Wander => MotionKind::Wander,
            Self::Chase => MotionKind::Chase,
            Self::ReturnToHome => MotionKind::ReturnToHome,
            Self::Flee => MotionKind::Flee,
            Self::KeepDistance(_) => MotionKind::KeepDistance,
            Self::Idle => MotionKind::Idle,
            Self::Orbit(_) => MotionKind::Orbit,
            Self::Dash => MotionKind::Dash,
            Self::Swoop => MotionKind::Swoop,
        }
    }
}

/// A [`Motion`] without its settings, for comparing motions and keying what each kind has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionKind {
    Wander,
    Chase,
    ReturnToHome,
    Flee,
    KeepDistance,
    Idle,
    Orbit,
    Dash,
    Swoop,
}

impl MotionKind {
    /// Whether this motion ends on its own, rather than running until replaced.
    pub const fn is_finite(self) -> bool {
        match self {
            Self::Dash | Self::Swoop => true,
            Self::Wander
            | Self::Chase
            | Self::ReturnToHome
            | Self::Flee
            | Self::KeepDistance
            | Self::Idle
            | Self::Orbit => false,
        }
    }

    /// Whether steering this motion needs a target.
    pub(super) fn needs_target(self) -> bool {
        match self {
            // Flee steers away from the target, so it needs one.
            Self::Chase
            | Self::KeepDistance
            | Self::Orbit
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
            Self::KeepDistance => "KeepDistance",
            Self::Idle => "Idle",
            Self::Orbit => "Orbit",
            Self::Dash => "Dash",
            Self::Swoop => "Swoop",
        }
    }
}

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
