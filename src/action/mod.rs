mod action_read;
mod in_progress;

use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::ActionSource;
    pub use super::action_read::{
        ActionCommands, ActionFailed, ActionFailure, ActionReach, ActionRead, ActionSlot, Attack,
        RequiredItem,
    };
    pub use super::in_progress::{
        ActionCooldown, ActionFinished, ActionGlow, ActionInProgress, ActionTookEffect,
        ActionUseStyle, ActionWindup, StopAction,
    };
}

/// Something a character uses through [`ActionCommands`]: one of its abilities or an equipped item.
/// Optional components beside it shape each use: [`ActionWindup`], [`ActionGlow`], [`ActionReach`],
/// [`ManaCost`], [`ManaDrainRate`] and [`RequiredItem`]. Its own [`ActionTookEffect`] observer
/// carries out the effect.
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
