mod stat_modifier;
mod status;

pub mod prelude {
    pub use super::stat_modifier::{Stat, StatMultipliers, stat_modifier};
    pub use super::status::{ApplyEffects, Burning, Effects, Frozen, Rallied};
}

use bevy::prelude::*;

#[cfg(feature = "dev_tools")]
pub(super) use self::status::StatusOf;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(status::on_apply_effects);

    app.add_systems(
        Update,
        (
            status::apply_burning,
            (status::tick_burn, status::while_burning).chain(),
            status::apply_frozen,
            status::apply_rallied,
            stat_modifier::sync_stat_multipliers_from_statuses,
        )
            .in_set(SimulationSystems::Upkeep),
    );
}
