use bevy::prelude::*;

use crate::{
    combat::{
        attributes::mana::*, damage::DamagePlugin, projectile::handle_collisions::*,
        status_effects::plugin::StatusEffectPlugin,
    },
    labels::sets::InGameSet,
};

use super::melee::swing_melee_attacks::update_melee_attacks;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DamagePlugin, StatusEffectPlugin))
            .add_systems(
                Update,
                (
                    regenerate_mana.in_set(InGameSet::Simulation),
                    update_melee_attacks.in_set(InGameSet::Simulation),
                    handle_projectile_collisions.in_set(InGameSet::Collision),
                ),
            );
    }
}
