mod issue_actions;

use bevy::prelude::*;

use super::BrainSystems;
use crate::prelude::*;

/// The one non-movement thing a character does at a time: use what it has in a slot, or
/// deliberately do nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Use the item equipped in a slot or one of the character's abilities.
    Use(ActionKey),
    /// Chose to do nothing, unlike an empty action slot where nothing scored.
    /// Always available; no applier carries it out.
    Idle,
}

impl Action {
    /// The key of the action source this action uses; `None` for `Idle`.
    pub fn key(self) -> Option<ActionKey> {
        match self {
            Self::Use(key) => Some(key),
            Self::Idle => None,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        issue_actions::issue_running_actions
            .in_set(BrainSystems::Execute)
            // Both Execute systems note what they issued on the `Brain`; they share one flush.
            .after_ignore_deferred(super::issue_motion::issue_running_motion),
    )
    .add_observer(issue_actions::on_action_finished);
}

/// A use by a character with a brain ended or could not start. See
/// [`Brain::finish_committed_behavior`](super::Brain::finish_committed_behavior) for what it ends.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(super) struct BehaviorActionFinished {
    pub entity: Entity,
    pub action: Action,
    pub outcome: ActionOutcome,
}

/// How an action reported in [`BehaviorActionFinished`] ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ActionOutcome {
    Completed,
    /// A committed use its applier could not start. A continuously scored use waits for
    /// its chance instead and never reports this.
    NotStarted,
}
