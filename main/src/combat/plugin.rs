use bevy::prelude::*;

use crate::{
    combat::{
        damage, health, invulnerable, mana, melee, projectile,
        status_effects::plugin::StatusEffectPlugin,
    },
    labels::sets::InGameSet,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatusEffectPlugin)
            .add_systems(
                Update,
                (
                    (
                        invulnerable::handle_invulnerability,
                        mana::regenerate_mana,
                        melee::process_melee_attacks,
                    )
                        .in_set(InGameSet::Simulation),
                    (
                        melee::end_melee_attacks,
                        projectile::handle_projectile_collisions,
                        melee::handle_melee_collisions,
                    )
                        .in_set(InGameSet::Collision),
                ),
            )
            .add_observer(health::on_healing_event)
            .add_observer(damage::on_damage_event);
    }
}
