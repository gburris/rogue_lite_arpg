mod action_status;
mod action_use;

use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::ActionSource;
    pub use super::action_status::{
        ActionCommands, ActionFailed, ActionFailure, ActionKey, ActionReach, ActionStatus,
        ActionUseStyle, Attack, RequiredItem,
    };
    pub use super::action_use::{
        ActionFinished, ActionGlow, ActionSourceUse, ActionTookEffect, ActionUseOf,
        ActionUsePartOf, ActionWindup, StopAction, TimedActionProgress,
    };
}

/// Something a character uses through [`ActionCommands`]: one of its abilities or an equipped item.
/// Optional components beside it, such as [`ActionWindup`] or [`ManaCost`], shape each use. Its own
/// [`ActionTookEffect`] observer carries out the effect.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(Cooldown, ActionUseStyle)]
pub struct ActionSource;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        action_use::advance_actions.in_set(SimulationSystems::Upkeep),
    )
    .add_observer(action_use::on_action_started)
    .add_observer(action_use::on_action_windup_started)
    .add_observer(action_use::on_stop_action);
}
