use bevy::prelude::*;

use super::{Brain, MotionCommands, MotionKey, TargetLocationHint};

/// What a brain holds in its motion slot: run one of the character's motion sources, or
/// deliberately stand still.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    /// Run the motion of the character's source with this key.
    Run(MotionKey),
    /// Chose to stand still, unlike an empty motion slot where nothing scored.
    /// Always available; it stops any running motion.
    Idle,
}

impl Motion {
    /// The key of the motion source this motion runs; `None` for `Idle`.
    pub fn key(self) -> Option<MotionKey> {
        match self {
            Self::Run(key) => Some(key),
            Self::Idle => None,
        }
    }

    /// One-word label for debug displays.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Run(key) => key.name(),
            Self::Idle => "Idle",
        }
    }
}

/// Issues the motion of the brain's motion behavior once, steering by the target's
/// [`LocationHint`](super::perception::LocationHint), and stops the running motion when the
/// motion slot is empty or idle. A new target brings a new hint, which retargets the running
/// motion. A finite motion already in progress is adopted, so its finish ends the new behavior.
pub(super) fn issue_running_motion(
    mut motions: MotionCommands,
    mut brains: Query<(Entity, &mut Brain, Option<&TargetLocationHint>)>,
) {
    for (entity, mut brain, location_hint) in &mut brains {
        let Some((behavior, motion)) = brain.current_motion_behavior() else {
            if motions.status().running_motion(entity).is_some() {
                motions.stop(entity);
            }
            continue;
        };
        let is_first_issue = !behavior.motion_issued;
        if is_first_issue {
            brain.record_issued_motion(behavior.behavior);
        }
        let Some(key) = motion.key() else {
            if motions.status().running_motion(entity).is_some() {
                motions.stop(entity);
            }
            continue;
        };

        let target = location_hint.map(TargetLocationHint::location_hint);

        if is_first_issue && !motions.status().is_in_progress(entity, key) {
            // `retarget` reads the running motion from before this start, so skip it.
            motions.start(entity, key, target);
            continue;
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
