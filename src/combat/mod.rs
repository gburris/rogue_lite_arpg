mod damage;
mod health;
mod invulnerable;
pub(crate) mod mana;
mod projectile;
mod status_effects;

use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::damage::{
        AttemptDamage, CombatSide, Damage, DamageDealt, Defeated, HurtBox, Knockback, hurtbox,
        hurtbox_at,
    };
    pub use super::health::{AttemptHeal, Healed, Health};
    pub use super::invulnerable::IFrames;
    pub use super::mana::{Mana, ManaCost, ManaDrainRate};
    pub use super::projectile::{
        FIREBALL_SPEED, FireProjectile, Projectile, ProjectileAttacker, Projectiles, fireball,
        fireball_with_speed, icebolt,
    };
    pub use super::status_effects::prelude::{Burning, Effects, Frozen};
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((projectile::plugin, status_effects::plugin));

    app.add_systems(
        Update,
        (
            (mana::regenerate_mana, mana::drain_mana_for_items_in_use)
                .chain()
                .in_set(SimulationSystems::Upkeep),
            // A flash ending resets the color to opaque white, so the flicker runs after it.
            (
                damage::tick_and_remove_damage_flash,
                invulnerable::tick_invulnerability,
            )
                .chain()
                .in_set(VfxSystems::Tint),
        ),
    )
    .add_observer(health::on_attempt_heal)
    .add_observer(damage::on_attempt_damage)
    .add_observer(damage::on_damage_dealt_flash)
    .add_observer(damage::on_damage_dealt_knockback);
}
