mod action_source;
mod action_use;

use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::action_source::{
        ActionCommands, ActionFailed, ActionFailure, ActionKey, ActionSource, ActionStatus,
        ActionUseStyle, Attack, RequiredItem,
    };
    pub use super::action_use::{
        ActionFinished, ActionGlow, ActionInUseOf, ActionSourceInUse, ActionTookEffect,
        ActionUsePartOf, ActionWindup, StopAction, TimedActionProgress,
    };
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        action_use::advance_actions.in_set(SimulationSystems::Upkeep),
    )
    .add_observer(action_use::on_action_started)
    .add_observer(action_use::on_action_windup_started)
    .add_observer(action_use::on_stop_action);
}
