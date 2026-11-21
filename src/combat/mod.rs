mod damage;
#[cfg(feature = "dev_tools")]
mod diagnostics;
mod health;
mod hitbox;
mod projectile;
mod status_effects;

use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    #[cfg(feature = "dev_tools")]
    pub use super::damage::Invincible;
    pub use super::damage::{AttemptDamage, CombatSide, Damage, DamageDealt, Defeated, Knockback};
    pub use super::health::{AttemptHeal, Healed, Health, IFrames, Mana, ManaCost, ManaDrainRate};
    pub use super::hitbox::{HitBox, HurtBox, LifeSteal, hurtbox, hurtbox_at};
    pub use super::projectile::{
        FIREBALL_SPEED, FireProjectiles, Projectile, ProjectileAttacker, ProjectileReflection,
        Projectiles, fireball, fireball_with_speed, icebolt, pull_back_from_walls,
    };
    pub use super::status_effects::prelude::{
        ApplyEffects, Effects, Frozen, Rallied, Stat, StatMultipliers, stat_modifier,
    };
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((projectile::plugin, status_effects::plugin));
    #[cfg(feature = "dev_tools")]
    app.add_plugins(diagnostics::plugin);

    app.add_systems(
        Update,
        (
            (
                (
                    health::regenerate_mana,
                    health::drain_mana_for_actions_in_progress,
                )
                    .chain(),
                projectile::reflect_projectiles,
            )
                .in_set(SimulationSystems::Upkeep),
            // A flash ending resets the color to opaque white, so the flicker runs after it.
            (
                damage::tick_and_remove_damage_flash,
                health::tick_invulnerability,
            )
                .chain()
                .in_set(VfxSystems::Tint),
            hitbox::damage_hitbox_contacts.in_set(InGameSystems::Collision),
        ),
    )
    .add_observer(health::on_attempt_heal)
    .add_observer(damage::on_attempt_damage)
    .add_observer(damage::on_damage_dealt_flash)
    .add_observer(damage::on_damage_dealt_knockback)
    .add_observer(hitbox::on_damage_dealt_life_steal);
}
