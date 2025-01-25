use bevy::prelude::*;

use crate::combat::{damage::DamagePlugin, status_effects::plugin::StatusEffectPlugin};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DamagePlugin, StatusEffectPlugin));
    }
}
