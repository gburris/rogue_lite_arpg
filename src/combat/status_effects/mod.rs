mod burn;
mod effect;
mod freeze;
mod slow;
mod status;

pub mod prelude {
    pub use super::burn::Burning;
    pub use super::effect::{ApplyEffects, Effects};
    pub use super::freeze::Frozen;
}

use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(effect::on_apply_effects);

    app.add_systems(
        Update,
        (
            burn::apply_burning,
            (burn::tick_burn, burn::while_burning).chain(),
            freeze::apply_frozen,
            slow::sync_slow_from_statuses,
        )
            .in_set(SimulationSystems::Upkeep),
    );
}
