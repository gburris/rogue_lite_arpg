//! One frame of a motion: what its system reads to steer, and how a finite
//! motion steps, follows its run, and reports finishing.

use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use super::{DecayingSpeedBoost, HomeArea, Motion, Steering};
use crate::character::brain::{Brain, Targeting};
use crate::character::state::Kinematics;
use crate::character::vision::TargetInfo;

/// A finite motion completed. The brain starts its cooldown and, if `behavior_run_id` is
/// still current, ends its behavior and decides again at once.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct MotionFinished {
    pub entity: Entity,
    pub motion: Motion,
    pub behavior_run_id: u32,
}

/// One frame of a finite motion.
pub(super) enum MotionStep {
    Steer(Steering),
    /// Ended this frame, maybe leaving a boost to ease off. The brain clears
    /// the slot when the finish flushes, so the next frame's run check resets
    /// the component before it could step the ended run again.
    Finished {
        decaying_speed_boost: Option<DecayingSpeedBoost>,
    },
}

/// The behavior run id a finite motion's component is serving, so a new run,
/// or the motion leaving the slot, resets the component's state.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CurrentBehaviorRunId {
    behavior_run_id: Option<u32>,
}

impl CurrentBehaviorRunId {
    /// Catches up with the brain's current run of `motion`; true when that run
    /// differs from the one followed so far.
    pub(super) fn follow_brain_run(&mut self, brain: &Brain, motion: Motion) -> bool {
        let behavior_run_id = brain
            .current_motion_behavior()
            .filter(|running| {
                running
                    .motion
                    .is_some_and(|running_motion| running_motion.index() == motion.index())
            })
            .map(|running| running.behavior_run_id);
        let changed = self.behavior_run_id != behavior_run_id;
        self.behavior_run_id = behavior_run_id;
        changed
    }

    /// The id of the followed run while its motion holds the slot.
    pub(super) fn get(self) -> Option<u32> {
        self.behavior_run_id
    }
}

/// Leaves any `decaying_speed_boost` to ease off and tells the brain the run is done.
pub(super) fn report_motion_finished(
    commands: &mut Commands,
    entity: Entity,
    motion: Motion,
    behavior_run_id: u32,
    decaying_speed_boost: Option<DecayingSpeedBoost>,
) {
    if let Some(decaying_speed_boost) = decaying_speed_boost {
        commands.entity(entity).try_insert(decaying_speed_boost);
    }
    commands.trigger(MotionFinished {
        entity,
        motion,
        behavior_run_id,
    });
}

/// What every motion system reads to steer a character.
#[derive(QueryData)]
pub(super) struct SteeredCharacter {
    pub(super) brain: &'static Brain,
    transform: &'static Transform,
    kinematics: &'static Kinematics,
    target_info: Option<&'static TargetInfo>,
    has_target: Has<Targeting>,
    home_area: Option<&'static HomeArea>,
}

impl SteeredCharacterItem<'_, '_> {
    /// This frame's input for steering `motion`, or `None` while it needs a
    /// target the character lacks.
    pub(super) fn build_steer_input(
        &self,
        motion: Motion,
        time: &Time<Virtual>,
    ) -> Option<SteerInput<'_>> {
        if motion.needs_target() && (!self.has_target || self.target_info.is_none()) {
            return None;
        }
        Some(SteerInput {
            position: self.transform.translation.xy(),
            heading: self.kinematics.direction,
            now: time.elapsed_secs(),
            delta_secs: time.delta_secs(),
            target_info: self.target_info,
            home_area: self.home_area,
            kinematics: self.kinematics,
        })
    }
}

pub(super) struct SteerInput<'a> {
    pub position: Vec2,
    pub heading: Vec2,
    pub now: f32,
    pub target_info: Option<&'a TargetInfo>,
    pub home_area: Option<&'a HomeArea>,
    delta_secs: f32,
    kinematics: &'a Kinematics,
}

impl SteerInput<'_> {
    pub(super) fn direction_to_target(&self) -> Option<Vec2> {
        let direction = self.target_info.map_or(Vec2::ZERO, |info| info.direction);
        (direction.length_squared() > f32::EPSILON).then_some(direction)
    }

    pub(super) fn direction_from_target(&self) -> Option<Vec2> {
        self.direction_to_target().map(|direction| -direction)
    }

    pub(super) fn direction_to_home(&self) -> Option<Vec2> {
        let home_area = self.home_area?;
        let to_home = home_area.origin() - self.position;
        (to_home.length() > home_area.arrival_radius()).then_some(to_home.normalize_or_zero())
    }

    /// How far the character covers this frame at `speed_multiplier` times
    /// max speed, after any slow.
    pub(super) fn distance_this_frame(&self, speed_multiplier: f32) -> f32 {
        self.kinematics.max_speed
            * speed_multiplier
            * self.kinematics.speed_scale()
            * self.delta_secs
    }
}
