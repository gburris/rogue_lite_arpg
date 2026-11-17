mod action_status;
mod in_progress;
mod source;

use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::ActionSource;
    pub use super::action_status::{ActionCommands, ActionFailed, ActionFailure, ActionStatus};
    pub use super::in_progress::{
        ActionFinished, ActionInProgress, ActionTookEffect, PartOfActionUse, StopAction,
        TimedActionProgress,
    };
    pub use super::source::{
        ActionCooldown, ActionGlow, ActionKey, ActionReach, ActionUseStyle, ActionWindup, Attack,
        RequiredItem,
    };
}

/// Something a character uses through [`ActionCommands`]: one of its abilities or an equipped item.
/// Optional components beside it, such as [`ActionWindup`] or [`ManaCost`], shape each use. Its own
/// [`ActionTookEffect`] observer carries out the effect.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(ActionCooldown, ActionUseStyle)]
pub struct ActionSource;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        in_progress::advance_actions.in_set(SimulationSystems::Upkeep),
    )
    .add_observer(in_progress::on_action_started)
    .add_observer(in_progress::on_action_windup_started)
    .add_observer(in_progress::on_stop_action);
}
