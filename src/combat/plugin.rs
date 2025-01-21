use bevy::prelude::*;

use crate::{combat::weapon::projectile_weapon::*, labels::sets::InGameSet};

use super::{
    damage::DamagePlugin, status_effects::plugin::StatusEffectPlugin,
    weapon::weapon::tick_weapon_attack_rate,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DamagePlugin, StatusEffectPlugin))
            .add_systems(
                Update,
                tick_weapon_attack_rate.in_set(InGameSet::Simulation),
            )
            .add_observer(on_weapon_attack);
    }
}
