pub mod damage;
pub mod health;
pub mod invulnerable;
pub mod mana;
pub mod melee;
pub mod projectile;
pub mod shield;
pub mod status_effects;

// These exist just to reduce stutter
pub use health::Health;
pub use mana::Mana;
pub use projectile::Projectile;

use bevy::prelude::*;

use crate::{combat::status_effects::StatusEffectPlugin, prelude::InGameSystems};

use shield::{
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
                        damage::tick_and_remove_damage_flash,
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
            .add_observer(damage::on_damage_dealt_flash)
            .add_observer(damage::on_damage_dealt_knockback)
            .add_observer(activate_shield);
    }
}
