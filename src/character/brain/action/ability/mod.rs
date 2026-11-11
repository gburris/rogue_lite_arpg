//! Innate abilities. Each is an opt-in capability component plus an applier that re-checks the
//! live world for the brain's held action and reports
//! [`ActionFinished`](crate::character::brain::ActionFinished). A defeated character loses its
//! brain, so appliers never see the dead.

mod ability_capabilities;
mod bite;
mod breath;
mod heal;

use bevy::prelude::*;

pub(in crate::character::brain) use self::ability_capabilities::AbilityCapabilities;
pub use self::bite::Bite;
pub use self::breath::Breath;
pub use self::heal::Heal;
pub(in crate::character::brain) use self::heal::find_carried_potion;
use crate::character::brain::BrainSystems;

/// An innate move a brain can ask its character for, carried out by a capability
/// component in this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ability {
    /// Drink a carried potion.
    Heal,
    /// Loose an innate projectile with no weapon.
    Breathe,
    /// Damage a target in contact reach.
    Bite,
}

impl Ability {
    /// Every ability, in array order.
    pub(in crate::character::brain) const ALL: [Self; 3] = [Self::Heal, Self::Breathe, Self::Bite];
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (heal::apply_heal, breath::apply_breath, bite::apply_bite).in_set(BrainSystems::Execute),
    );
}
