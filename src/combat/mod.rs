mod damage;
mod health;
mod invulnerable;
mod mana;
mod projectile;
mod status_effects;

use bevy::prelude::*;

use crate::prelude::InGameSystems;

pub mod prelude {
    pub use super::damage::*;
    pub use super::health::*;
    pub use super::invulnerable::*;
    pub use super::mana::*;
    pub use super::projectile::*;
    pub use super::status_effects::prelude::*;
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
