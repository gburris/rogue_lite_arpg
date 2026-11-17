use bevy::prelude::*;

use super::{Brain, Motion, MotionCommands, RunningBehavior, TargetingData};

/// The running behavior the brain last started a motion for, so each behavior starts its motion
/// once and a finished motion can name its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct StartedMotion {
    behavior: Option<RunningBehavior>,
}

impl StartedMotion {
    /// The run id of the behavior a finished `motion` belongs to; `None` if the brain has since
    /// started a different kind of motion.
    pub(super) fn behavior_run_id_of(&self, motion: Motion) -> Option<u32> {
        self.behavior
            .filter(|behavior| {
                behavior
                    .motion
                    .is_some_and(|started| started.is_same_kind(motion))
            })
            .map(|behavior| behavior.behavior_run_id)
    }

    fn behavior_run_id(&self) -> Option<u32> {
        self.behavior.map(|behavior| behavior.behavior_run_id)
    }
}

/// Starts the motion of the brain's motion behavior once per running behavior, keeps its target
/// current, and stops it when the motion slot is empty. A finite motion already in progress is
/// adopted, so its finish ends the new behavior.
pub(super) fn apply_motion_requests(
    mut motions: MotionCommands,
    mut brains: Query<(Entity, &Brain, TargetingData, &mut StartedMotion)>,
) {
    for (entity, brain, targeting, mut started_motion) in &mut brains {
        let Some((behavior, motion)) = brain.current_motion_behavior() else {
            if started_motion.behavior.take().is_some() {
                motions.stop(entity);
            }
            continue;
        };

        let target = if motion.needs_target() {
            targeting.motion_target()
        } else {
            None
        };

        if started_motion.behavior_run_id() != Some(behavior.behavior_run_id) {
            started_motion.behavior = Some(behavior);
            if !motions.status().is_in_progress(entity, motion) {
                // `retarget` reads the running motion from before this start, so skip it.
                motions.start(entity, motion, target);
                continue;
            }
        }

        let running_target = motions
            .status()
            .running_motion(entity)
            .and_then(|running_motion| running_motion.target);
        if running_target != target {
            motions.retarget(entity, target);
        }
    }
}
