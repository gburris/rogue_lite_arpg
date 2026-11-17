mod usage;

use bevy::prelude::*;

pub(super) use self::usage::StartedActions;
use super::BrainSystems;
use crate::prelude::*;

/// The one non-movement thing a character does at a time: use what it has in a slot, or
/// deliberately do nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Use the item equipped in a slot or one of the character's abilities.
    Use(ActionSlot),
    /// Chose to do nothing, unlike an empty action slot where nothing scored.
    /// Always available; no applier carries it out.
    Idle,
}

impl Action {
    /// The slot this action uses; `None` for `Idle`.
    pub fn slot(self) -> Option<ActionSlot> {
        match self {
            Self::Use(slot) => Some(slot),
            Self::Idle => None,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        usage::apply_action_requests.in_set(BrainSystems::Execute),
    )
    .add_observer(usage::on_action_finished);
}

/// A use ended. It ends the behavior stamped `behavior_run_id` only while that behavior still
/// runs, so a stale finish clears nothing.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub(super) struct BehaviorActionFinished {
    pub entity: Entity,
    pub action: Action,
    pub behavior_run_id: u32,
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
