use bevy::prelude::*;

use super::{Brain, Motion, MotionCommands, RunningBehavior, TargetLocationHint};

/// The running behavior the brain last issued a motion for, by starting or adopting it, so each
/// behavior issues its motion once and a finished motion can name its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct IssuedMotion {
    behavior: Option<RunningBehavior>,
}

impl IssuedMotion {
    /// The id of the running behavior a finished `motion` belongs to; `None` if the brain has
    /// since issued a different motion.
    pub(super) fn running_behavior_id_of(&self, motion: Motion) -> Option<u32> {
        self.behavior
            .filter(|behavior| behavior.motion.is_some_and(|started| started == motion))
            .map(|behavior| behavior.running_behavior_id)
    }

    fn running_behavior_id(&self) -> Option<u32> {
        self.behavior.map(|behavior| behavior.running_behavior_id)
    }

    fn record_issued_motion(&mut self, running: RunningBehavior) {
        self.behavior = Some(running);
    }
}

/// Starts the motion of the brain's motion behavior once per running behavior, steering by the
/// target's [`LocationHint`](super::perception::LocationHint), and stops it when the motion slot
/// is empty. A new target brings a new hint, which retargets the running motion. A finite motion
/// already in progress is adopted, so its finish ends the new behavior.
pub(super) fn issue_running_motion(
    mut motions: MotionCommands,
    mut brains: Query<(
        Entity,
        &Brain,
        Option<&TargetLocationHint>,
        &mut IssuedMotion,
    )>,
) {
    for (entity, brain, location_hint, mut issued_motion) in &mut brains {
        let Some((behavior, motion)) = brain.current_motion_behavior() else {
            if issued_motion.behavior.take().is_some() {
                motions.stop(entity);
            }
            continue;
        };

        let target = location_hint.map(TargetLocationHint::location_hint);

        if issued_motion.running_behavior_id() != Some(behavior.running_behavior_id) {
            issued_motion.record_issued_motion(behavior);
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
