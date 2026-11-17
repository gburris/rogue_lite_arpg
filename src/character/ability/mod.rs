mod abilities;
mod bite;
mod breath;
mod heal;
mod rally;

use bevy::prelude::*;

pub use self::abilities::Abilities;
pub use self::bite::{Bite, bite};
pub use self::breath::breath;
pub use self::heal::heal;
pub use self::rally::speed_and_damage_rally;
use crate::prelude::*;

/// An innate move a character can use, carried out by a capability component of the same name,
/// such as [`Bite`], on one of its [`Abilities`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[require(ActionSource)]
pub enum Ability {
    /// Drink a carried potion.
    Heal,
    /// Loose an innate projectile with no weapon.
    Breath,
    /// Damage a target in contact reach.
    Bite,
    /// Raise the stats of every ally in reach with a war cry.
    Rally,
}

impl Ability {
    /// Every ability.
    pub const ALL: [Self; 4] = [Self::Heal, Self::Breath, Self::Bite, Self::Rally];
}

// A new variant fails to compile here: add it to this match and to `Ability::ALL`.
const _: () = match Ability::Heal {
    Ability::Heal | Ability::Breath | Ability::Bite | Ability::Rally => {}
};
