mod dash;
mod direct;
mod home_area;
mod motion_source;
mod orbit;
mod running_motion;
mod steer_input;
mod steering;
mod swoop;
mod wander;

use bevy::prelude::*;

pub use self::dash::Dash;
pub use self::direct::KeepDistance;
pub use self::home_area::HomeArea;
use self::motion_source::TunedBy;
pub use self::motion_source::{MotionReach, MotionSources};
pub use self::orbit::Orbit;
pub use self::running_motion::CurrentMotion;
pub(super) use self::running_motion::{MotionCommands, MotionFinished, MotionStatus};
use self::running_motion::{MotionOf, RunningMotion};
pub(super) use self::steer_input::AssumedVelocity;
use self::steer_input::{
    BlockedTimer, MotionStep, SteerInput, SteeredCharacter, SteeringTarget, SteeringTargets,
};
pub use self::steering::TurnRate;
use self::steering::{DecayingSpeedBoost, Steering};
pub use self::swoop::Swoop;
pub use self::wander::{Wander, WanderProgress};
use crate::prelude::*;

/// How a character moves: a continuous motion that runs until replaced, or a finite motion that
/// ends on its own. Add a variant only for a new kind of steering. On a motion source, it names the
/// motion the source tunes.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Motion {
    /// Alternates pausing and walking; see [`Wander`].
    #[default]
    Wander,
    Chase,
    ReturnToHome,
    /// Moves directly away from the target.
    Flee,
    /// See [`KeepDistance`].
    KeepDistance,
    /// Stands still. Unlike an empty motion slot, it is scored and chosen like any other motion.
    Idle,
    /// See [`Orbit`].
    Orbit,
    /// See [`Dash`].
    Dash,
    /// See [`Swoop`].
    Swoop,
}

impl Motion {
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

    /// Whether a character needs a motion source of this motion to start it, because the motion
    /// reads its tuning from the source.
    pub(super) fn needs_source(self) -> bool {
        match self {
            Self::Wander | Self::KeepDistance | Self::Orbit | Self::Dash | Self::Swoop => true,
            Self::Chase | Self::ReturnToHome | Self::Flee | Self::Idle => false,
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
    app.add_observer(dash::on_dash_started)
        .add_observer(swoop::on_swoop_started)
        .add_observer(orbit::on_orbit_started)
        .add_observer(wander::on_wander_started)
        .add_observer(home_area::on_return_to_home_started);
}
