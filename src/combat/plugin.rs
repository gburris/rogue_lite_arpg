use bevy::prelude::*;

use crate::{
    combat::{
        damage, health, invulnerable, mana, melee, projectile, status_effects::StatusEffectPlugin,
    },
    labels::sets::InGameSystems,
};

use super::shield::{
    handle_collisions::handle_projectile_reflection_collisions,
    shield_block::{activate_shield, update_active_shields},
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
                        update_active_shields,
                    )
                        .in_set(InGameSystems::Simulation),
                    (
                        melee::end_melee_attacks,
                        handle_projectile_reflection_collisions,
                    )
                        .in_set(InGameSystems::Simulation),
                    (
                        projectile::handle_collisions,
                        melee::handle_melee_collisions,
                    )
                        .in_set(InGameSystems::Collision),
                ),
            )
            .add_observer(health::on_healing_event)
            .add_observer(damage::on_damage_event)
            .add_observer(activate_shield);
    }
}
