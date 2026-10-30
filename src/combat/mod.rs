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
    pub(crate) use super::mana::regenerate_mana;
    pub use super::mana::{Mana, ManaCost, ManaDrainRate};
    pub use super::projectile::{
        FIREBALL_SPEED, FireProjectile, Projectile, ProjectileAttacker, Projectiles, fireball,
        fireball_with_speed, icebolt,
    };
    pub(crate) use super::status_effects::prelude::sync_slow_from_statuses;
    pub use super::status_effects::prelude::{Burning, Effects, Frozen};
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((projectile::plugin, status_effects::plugin));

    app.add_systems(
        Update,
        ((
            invulnerable::handle_invulnerability,
            mana::regenerate_mana,
            damage::tick_and_remove_damage_flash,
        )
            .in_set(InGameSystems::Simulation),),
    )
    .add_observer(health::on_healing_event)
    .add_observer(damage::on_damage_event)
    .add_observer(damage::on_damage_dealt_flash)
    .add_observer(damage::on_damage_dealt_knockback);
}
