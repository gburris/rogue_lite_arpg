mod in_use;
mod use_read;

use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::Usable;
    pub use super::in_use::{
        InUse, StopUse, UseCooldown, UseFinished, UseGlow, UseStyle, UseTookEffect, UseWindup,
    };
    pub use super::use_read::{
        RequiredItem, UseCommands, UseFailed, UseFailure, UseReach, UseRead, UseSlot,
    };
}

/// Something a character uses through [`UseCommands`]: one of its abilities or an equipped item.
/// Optional components beside it shape each use: [`UseWindup`], [`UseGlow`], [`UseReach`],
/// [`ManaCost`], [`ManaDrainRate`] and [`RequiredItem`]. Its own [`UseTookEffect`] observer
/// carries out the effect.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(UseCooldown, UseStyle)]
pub struct Usable;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        in_use::advance_uses.in_set(SimulationSystems::Upkeep),
    )
    .add_observer(in_use::on_use_started)
    .add_observer(in_use::on_use_windup_started)
    .add_observer(in_use::on_stop_use);
}
