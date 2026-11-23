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

pub use self::dash::{kite, sidestep, slide, step_in};
pub use self::direct::{Chase, Flee, Idle, KeepDistance};
pub use self::home_area::{Home, HomeArea, ReturnToHome};
use self::motion_source::{FiniteMotion, MotionSourceOf};
pub use self::motion_source::{MotionKey, MotionSources};
pub use self::orbit::Orbit;
pub use self::running_motion::RunningMotion;
pub(super) use self::running_motion::{MotionCommands, MotionFailed, MotionFinished, MotionStatus};
use self::running_motion::{MotionProgress, MotionStep, RunningMotionOf, SteeringMotionOf};
use self::steer_input::{
    BlockedTimer, CircleDirection, SteerInput, SteeredCharacter, SteeringTarget, SteeringTargets,
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
            running_motion::apply_motion::<dash::DashProgress>,
            running_motion::apply_motion::<swoop::SwoopProgress>,
            running_motion::apply_motion::<orbit::OrbitProgress>,
            running_motion::apply_motion::<WanderProgress>,
            home_area::move_homes_of_blocked_characters,
            running_motion::apply_motion::<home_area::ReturnToHomeProgress>,
            direct::apply_direct_steering::<Idle>,
            direct::apply_direct_steering::<Chase>,
            direct::apply_direct_steering::<Flee>,
            direct::apply_direct_steering::<KeepDistance>,
            steering::decay_speed_boosts,
            steering::apply_steering,
        )
            .chain()
            .in_set(SimulationSystems::Steer),
    );
    app.add_observer(motion_source::on_motion_source_added)
        .add_observer(running_motion::on_motion_started::<dash::DashProgress>)
        .add_observer(running_motion::on_motion_started::<swoop::SwoopProgress>)
        .add_observer(running_motion::on_motion_started::<orbit::OrbitProgress>)
        .add_observer(running_motion::on_motion_started::<WanderProgress>)
        .add_observer(running_motion::on_motion_started::<home_area::ReturnToHomeProgress>);
}
