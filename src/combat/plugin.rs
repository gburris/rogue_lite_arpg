use bevy::prelude::*;

use crate::{
    combat::{
        attributes::mana::*, damage::DamagePlugin, projectile::handle_collisions::*,
        status_effects::plugin::StatusEffectPlugin,
    },
    labels::sets::InGameSet,
};

use super::melee::{
    handle_collisions::handle_melee_collisions,
    swing_melee_attacks::{end_melee_attacks, process_melee_attacks},
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DamagePlugin, StatusEffectPlugin))
            .add_systems(
                Update,
                (
                    regenerate_mana.in_set(InGameSet::Simulation),
                    process_melee_attacks.in_set(InGameSet::Simulation),
                    end_melee_attacks.in_set(InGameSet::Collision),
                    handle_projectile_collisions.in_set(InGameSet::Collision),
                    handle_melee_collisions.in_set(InGameSet::Collision),
                ),
            );
    }
}
