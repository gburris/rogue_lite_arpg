mod burn;
mod effect;
mod freeze;
mod rally;
mod stat_modifier;
mod status;

pub mod prelude {
    pub use super::burn::Burning;
    pub use super::effect::{ApplyEffects, Effects};
    pub use super::freeze::Frozen;
    pub use super::rally::Rallied;
    pub use super::stat_modifier::{Stat, StatMultipliers, stat_modifier};
}

use bevy::prelude::*;

#[cfg(feature = "dev_tools")]
pub(super) use self::status::StatusOf;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(effect::on_apply_effects);

    app.add_systems(
        Update,
        (
            burn::apply_burning,
            (burn::tick_burn, burn::while_burning).chain(),
            freeze::apply_frozen,
            rally::apply_rallied,
            stat_modifier::sync_stat_multipliers_from_statuses,
        )
            .in_set(SimulationSystems::Upkeep),
    );
}
