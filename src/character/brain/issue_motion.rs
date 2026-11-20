use bevy::prelude::*;

use super::{Brain, Motion, MotionCommands, TargetLocationHint};

/// Keeps each character's running motion on the motion its brain holds, steering by the target's
/// [`LocationHint`](super::perception::LocationHint), and stops it when the motion slot is empty or
/// idle; see [`MotionCommands::start`]. A finished motion is not started again; see
/// [`on_motion_finished`](super::decide::on_motion_finished).
pub(super) fn issue_running_motion(
    mut motions: MotionCommands,
    brains: Query<(Entity, &Brain, Option<&TargetLocationHint>)>,
) {
    for (entity, brain, location_hint) in &brains {
        let Some(key) = brain.motion().and_then(Motion::key) else {
            motions.stop(entity);
            continue;
        };
        let target = location_hint.map(TargetLocationHint::location_hint);
        motions.start(entity, key, target);
    }
}
