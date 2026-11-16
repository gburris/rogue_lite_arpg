use bevy::prelude::*;

use super::{Action, ActionOutcome, BehaviorActionFinished};
use crate::character::brain::Brain;
use crate::prelude::*;

/// The behavior run id of the last use started in each slot, so a committed action starts once
/// and its finish names its behavior.
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub(in crate::character::brain) struct StartedActions([Option<u32>; ActionSlot::COUNT]);

/// Starts the actions of the brain's running behaviors and stops sustained actions none of them
/// still chooses. Considerations already judged the target, so the game decides only whether a use
/// can start. A committed action starts once, or adopts a use of it already in progress so that
/// use's finish ends the behavior; a continuously scored one starts again whenever it isn't in
/// progress.
pub(super) fn apply_action_requests(
    mut commands: Commands,
    mut actions: ActionCommands,
    mut brains: Query<(Entity, &Brain, &mut StartedActions)>,
) {
    for (entity, brain, mut started_actions) in &mut brains {
        for slot in ActionSlot::ALL {
            if brain.running_behavior_with(Action::Use(slot)).is_none() {
                actions.stop_sustained_use(entity, slot);
            }
        }
        for &running in &brain.running_behaviors {
            let slot = match running.action {
                Some(Action::Use(slot)) => slot,
                Some(Action::Idle) | None => continue,
            };
            let started_behavior_run_id = &mut started_actions[slot.index()];
            if running.committed && *started_behavior_run_id == Some(running.behavior_run_id) {
                continue;
            }
            if actions.read().is_using(entity, slot) {
                if running.committed {
                    *started_behavior_run_id = Some(running.behavior_run_id);
                }
                continue;
            }
            if actions.try_use(entity, slot).is_ok() {
                *started_behavior_run_id = Some(running.behavior_run_id);
            } else {
                running.report_not_started(&mut commands, entity);
            }
        }
    }
}

/// Reports [`BehaviorActionFinished`] for the use that finished.
pub(super) fn on_action_finished(
    finished: On<ActionFinished>,
    mut commands: Commands,
    started_actions: Query<&StartedActions>,
) {
    let Some(behavior_run_id) = started_actions
        .get(finished.holder)
        .ok()
        .and_then(|started_actions| started_actions[finished.slot.index()])
    else {
        return;
    };
    commands.trigger(BehaviorActionFinished {
        entity: finished.holder,
        action: Action::Use(finished.slot),
        behavior_run_id,
        outcome: ActionOutcome::Completed,
    });
}
