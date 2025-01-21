use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{
    damage::DamagePlugin, status_effects::plugin::StatusEffectPlugin,
    weapon::weapon::tick_equippable_use_rate,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DamagePlugin, StatusEffectPlugin))
            .add_systems(
                Update,
                tick_equippable_use_rate.in_set(InGameSet::Simulation),
            );
    }
}
