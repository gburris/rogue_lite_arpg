mod stat_modifier;
mod status;

use bevy::prelude::*;

pub use self::stat_modifier::{Stat, StatMultipliers, stat_modifier};
#[cfg(feature = "dev_tools")]
pub use self::status::StatusOf;
pub use self::status::{ApplyEffects, Burning, Effects, Frozen, Rallied};
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
