use bevy::prelude::*;

use crate::{
    combat::status_effects::{handle_statuses::*, status_systems::*},
    labels::sets::InGameSet,
};

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_effect_apply)
            .add_observer(on_status_apply)
            .add_systems(
                Update,
                (burning::tick_burn, burning::while_burning)
                    .chain()
                    .in_set(InGameSet::Simulation),
            )
            .add_observer(burning::on_burn_applied)
            .add_observer(burning::on_burn_removed)
            .add_observer(frozen::on_frozen_applied)
            .add_observer(frozen::on_frozen_removed)
            .add_observer(slowed::on_slow_applied)
            .add_observer(slowed::on_slow_removed)
            .add_observer(stunned::on_stun_applied)
            .add_observer(stunned::on_stun_removed);
    }
}
