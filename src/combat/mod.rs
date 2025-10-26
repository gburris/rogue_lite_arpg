pub mod damage;
pub mod health;
pub mod invulnerable;
pub mod mana;
pub mod projectile;
pub mod status_effects;

// These exist just to reduce stutter
pub use health::Health;
pub use mana::Mana;
pub use projectile::Projectile;

use bevy::prelude::*;

use crate::{combat::status_effects::StatusEffectPlugin, prelude::InGameSystems};

pub mod prelude {
    pub use super::mana::*;
    pub use super::projectile::*;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((projectile::plugin, StatusEffectPlugin));

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
