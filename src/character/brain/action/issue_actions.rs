use bevy::prelude::*;

use super::{Action, ActionOutcome, BehaviorActionFinished};
use crate::character::brain::Brain;
use crate::prelude::*;

/// Issues the actions of the brain's running behaviors and stops sustained actions none of them
/// still chooses. Considerations already judged the target, so the game decides only whether a use
/// can start. A committed action is issued once, or adopts a use of it already in progress so that
/// use's finish ends the behavior; a continuously scored one starts again whenever it isn't in
/// progress.
pub(super) fn issue_running_actions(
    mut commands: Commands,
    mut actions: ActionCommands,
    mut brains: Query<(Entity, &mut Brain)>,
) {
    for (entity, mut brain) in &mut brains {
        for key in ActionKey::all() {
            if brain.running_behavior_with(Action::Use(key)).is_none() {
                actions.stop_sustained_use(entity, key);
            }
        }
        for index in 0..brain.running_behaviors.len() {
            let running = brain.running_behaviors[index];
            let Some(key) = running.action.and_then(Action::key) else {
                continue;
            };
            if running.committed {
                if running.action_issued {
                    continue;
                }
                brain.record_issued_action(running.behavior);
            }
            if actions.status().is_using(entity, key) {
                continue;
            }
            if actions.try_use(entity, key).is_err() {
                running.report_not_started(&mut commands, entity);
            }
        }
    }
}

/// Reports [`BehaviorActionFinished`] for a use that finished on a character with a brain.
pub(super) fn on_action_finished(
    finished: On<ActionFinished>,
    mut commands: Commands,
    brains: Query<(), With<Brain>>,
) {
    if !brains.contains(finished.holder) {
        return;
    }
    commands.trigger(BehaviorActionFinished {
        entity: finished.holder,
        action: Action::Use(finished.key),
        outcome: ActionOutcome::Completed,
    });
}
