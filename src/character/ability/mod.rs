mod ability_use;
mod bite;
mod breath;
mod heal;
mod rally;

use bevy::prelude::*;

pub use self::ability_use::{
    Abilities, AbilityCooldown, AbilityRead, AbilityStep, AbilitySteps, AbilityUsage,
    AbilityUseFailure, AbilityUseFinished, UseGlow,
};
use self::ability_use::{AbilityOf, AbilityReleased};
pub use self::bite::{Bite, bite};
pub use self::breath::breath;
pub use self::heal::heal;
pub use self::rally::speed_and_damage_rally;
use crate::prelude::*;

/// An innate move a character can use, carried out by a capability component
/// ([`Heal`](self::heal::Heal), [`Breath`](self::breath::Breath), [`Bite`] or
/// [`Rally`](self::rally::Rally)) on one of its [`Abilities`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ability {
    /// Drink a carried potion.
    Heal,
    /// Loose an innate projectile with no weapon.
    Breathe,
    /// Damage a target in contact reach.
    Bite,
    /// Raise the stats of every ally in reach with a war cry.
    Rally,
}

impl Ability {
    /// Every ability, in array order.
    pub const ALL: [Self; 4] = [Self::Heal, Self::Breathe, Self::Bite, Self::Rally];
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        ability_use::advance_ability_uses.in_set(SimulationSystems::Upkeep),
    )
    .add_systems(
        Update,
        bite::damage_bite_hits.in_set(InGameSystems::Collision),
    )
    .add_observer(ability_use::on_ability_use_started);
}
