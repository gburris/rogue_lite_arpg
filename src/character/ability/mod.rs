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

/// An innate move a character can use, carried out by a capability component
/// ([`Heal`](self::heal::Heal), [`Breath`](self::breath::Breath), [`Bite`] or
/// [`Rally`](self::rally::Rally)) on one of its [`Abilities`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
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
    /// Every ability, in array order.
    pub const ALL: [Self; 4] = [Self::Heal, Self::Breath, Self::Bite, Self::Rally];
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        bite::damage_bite_hits.in_set(InGameSystems::Collision),
    );
}
