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
use self::direct::DirectMotion;
pub use self::direct::{Chase, Flee, KeepDistance};
pub use self::home_area::{HomeArea, ReturnToHome};
use self::motion_source::{FiniteMotion, TunedBy};
pub use self::motion_source::{MotionKey, MotionReach, MotionSources};
pub use self::orbit::Orbit;
pub use self::running_motion::CurrentMotion;
pub(super) use self::running_motion::{MotionCommands, MotionFinished, MotionStatus};
use self::running_motion::{MotionOf, MotionProgress, RunningMotion};
pub(super) use self::steer_input::AssumedVelocity;
use self::steer_input::{
    BlockedTimer, CircleDirection, MotionStep, SteerInput, SteeredCharacter, SteeringTarget,
    SteeringTargets,
};
pub use self::steering::TurnRate;
use self::steering::{DecayingSpeedBoost, Steering};
pub use self::swoop::Swoop;
pub use self::wander::{Wander, WanderProgress};
use crate::prelude::*;

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
            direct::apply_direct_steering::<Chase>,
            direct::apply_direct_steering::<Flee>,
            direct::apply_direct_steering::<KeepDistance>,
            direct::apply_direct_steering::<ReturnToHome>,
            steering::decay_speed_boosts,
            steering::apply_steering,
        )
            .chain()
            .in_set(SimulationSystems::Steer),
    );
    app.add_observer(running_motion::on_motion_started::<dash::DashProgress>)
        .add_observer(running_motion::on_motion_started::<swoop::SwoopProgress>)
        .add_observer(running_motion::on_motion_started::<orbit::OrbitProgress>)
        .add_observer(running_motion::on_motion_started::<WanderProgress>)
        .add_observer(running_motion::on_motion_started::<home_area::ReturnToHomeProgress>);
}
