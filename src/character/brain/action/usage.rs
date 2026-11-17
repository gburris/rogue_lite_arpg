use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use super::{Action, ActionOutcome, BehaviorActionFinished};
use crate::character::brain::{Brain, RunningBehavior};
use crate::prelude::*;

/// The behavior run id of the last use started by each key, so a committed action starts once
/// and its finish names its behavior.
#[derive(Component, Clone, Debug, Default)]
pub(in crate::character::brain) struct StartedActions(HashMap<ActionKey, u32>);

impl StartedActions {
    fn behavior_run_id(&self, key: ActionKey) -> Option<u32> {
        self.0.get(&key).copied()
    }

    fn was_started_by(&self, key: ActionKey, running: RunningBehavior) -> bool {
        running.committed && self.behavior_run_id(key) == Some(running.behavior_run_id)
    }

    fn record_start(&mut self, key: ActionKey, running: RunningBehavior) {
        self.0.insert(key, running.behavior_run_id);
    }
}

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
        for key in ActionKey::all() {
            if brain.running_behavior_with(Action::Use(key)).is_none() {
                actions.stop_sustained_use(entity, key);
            }
        }
        for &running in &brain.running_behaviors {
            let Some(key) = running.action.and_then(Action::key) else {
                continue;
            };
            if started_actions.was_started_by(key, running) {
                continue;
            }
            if actions.read().is_using(entity, key) {
                if running.committed {
                    started_actions.record_start(key, running);
                }
                continue;
            }
            if actions.try_use(entity, key).is_ok() {
                started_actions.record_start(key, running);
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
        .and_then(|started_actions| started_actions.behavior_run_id(finished.key))
    else {
        return;
    };
    commands.trigger(BehaviorActionFinished {
        entity: finished.holder,
        action: Action::Use(finished.key),
        behavior_run_id,
        outcome: ActionOutcome::Completed,
    });
}
