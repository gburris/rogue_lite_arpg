mod damage;
#[cfg(feature = "dev_tools")]
mod diagnostics;
mod health;
#[cfg(feature = "dev_tools")]
mod invincible;
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
    #[cfg(feature = "dev_tools")]
    pub use super::invincible::Invincible;
    pub use super::invulnerable::IFrames;
    pub use super::mana::{Mana, ManaCost, ManaDrainRate};
    pub use super::projectile::{
        FIREBALL_SPEED, FireProjectile, Projectile, ProjectileAttacker, ProjectileReflection,
        Projectiles, fireball, fireball_with_speed, icebolt,
    };
    pub use super::status_effects::prelude::{Effects, Frozen, MovementSlow};
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((projectile::plugin, status_effects::plugin));
    #[cfg(feature = "dev_tools")]
    app.add_plugins(diagnostics::plugin);

    app.add_systems(
        Update,
        (
            (
                (mana::regenerate_mana, mana::drain_mana_for_items_in_use).chain(),
                projectile::reflect_projectiles,
            )
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
