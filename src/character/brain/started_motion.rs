use bevy::prelude::*;

use super::motion::{MotionCommands, TargetRef};
use super::{Brain, Motion, RunningBehavior, TargetingData};

/// The behavior run the brain last started a motion for and the target it gave it, so each run
/// starts its motion once, a new target reaches it, and a finished motion names its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct StartedMotion {
    running: Option<RunningBehavior>,
    target: Option<TargetRef>,
}

impl StartedMotion {
    /// The run a finished `motion` belongs to; `None` once the brain has started another kind of
    /// motion since.
    pub(super) fn behavior_run_id_of(&self, motion: Motion) -> Option<u32> {
        self.running
            .filter(|running| running.motion.is_some_and(|held| held.is_same_kind(motion)))
            .map(|running| running.behavior_run_id)
    }

    fn behavior_run_id(&self) -> Option<u32> {
        self.running.map(|running| running.behavior_run_id)
    }
}

/// Starts the motion of the brain's motion behavior once per run, keeps it pointed at the target,
/// and stops it once no behavior holds the motion slot. A finite motion still in progress from an
/// earlier run is adopted, so its finish ends the new run.
pub(super) fn apply_motion_requests(
    mut motions: MotionCommands,
    mut brains: Query<(Entity, &Brain, TargetingData, &mut StartedMotion)>,
) {
    for (entity, brain, targeting, mut started_motion) in &mut brains {
        let Some(running) = brain.current_motion_behavior() else {
            if started_motion.running.take().is_some() {
                motions.stop(entity);
                started_motion.target = None;
            }
            continue;
        };
        let Some(motion) = running.motion else {
            continue;
        };
        let target = if motion.needs_target() {
            targeting.motion_target()
        } else {
            None
        };
        if started_motion.behavior_run_id() != Some(running.behavior_run_id) {
            if !(motion.is_finite() && motions.status().is_in_progress(entity, motion)) {
                motions.start(entity, motion, target);
                started_motion.target = target;
            }
            started_motion.running = Some(running);
        }
        if started_motion.target != target {
            motions.retarget(entity, target);
            started_motion.target = target;
        }
    }
}
