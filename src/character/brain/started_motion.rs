use bevy::prelude::*;

use super::{Brain, MotionCommands, MotionKind, RunningBehavior, TargetLocationHint};

/// The running behavior the brain last started a motion for, so each behavior starts its motion
/// once and a finished motion can name its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct StartedMotion {
    behavior: Option<RunningBehavior>,
}

impl StartedMotion {
    /// The run id of the behavior a finished motion of `kind` belongs to; `None` if the brain has
    /// since started a different kind of motion.
    pub(super) fn behavior_run_id_of(&self, kind: MotionKind) -> Option<u32> {
        self.behavior
            .filter(|behavior| {
                behavior
                    .motion
                    .is_some_and(|started| started.kind() == kind)
            })
            .map(|behavior| behavior.behavior_run_id)
    }

    fn behavior_run_id(&self) -> Option<u32> {
        self.behavior.map(|behavior| behavior.behavior_run_id)
    }
}

/// Starts the motion of the brain's motion behavior once per running behavior, steering by the
/// target's [`LocationHint`](super::perception::LocationHint), and stops it when the motion slot
/// is empty. A new target brings a new hint, which retargets the running motion. A finite motion
/// already in progress is adopted, so its finish ends the new behavior.
pub(super) fn apply_motion_requests(
    mut motions: MotionCommands,
    mut brains: Query<(
        Entity,
        &Brain,
        Option<&TargetLocationHint>,
        &mut StartedMotion,
    )>,
) {
    for (entity, brain, location_hint, mut started_motion) in &mut brains {
        let Some((behavior, motion)) = brain.current_motion_behavior() else {
            if started_motion.behavior.take().is_some() {
                motions.stop(entity);
            }
            continue;
        };

        let target = location_hint.map(TargetLocationHint::location_hint);

        if started_motion.behavior_run_id() != Some(behavior.behavior_run_id) {
            started_motion.behavior = Some(behavior);
            if !motions.status().is_in_progress(entity, motion.kind()) {
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
