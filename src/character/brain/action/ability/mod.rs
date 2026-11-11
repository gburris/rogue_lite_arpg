//! Innate abilities. Each is an opt-in capability component, read by the
//! decision's `DecisionInputs::action_profile`, plus an applier that re-checks the live world for
//! the brain's held action and reports
//! [`ActionFinished`](crate::character::brain::ActionFinished). Appliers never change the
//! motion, and a defeated character loses its brain, so they never see the dead.

mod bite;
mod breath;
mod heal;

use bevy::prelude::*;

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
    pub const ALL: [Self; 3] = [Self::Heal, Self::Breathe, Self::Bite];
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (heal::apply_heal, breath::apply_breath, bite::apply_bite).in_set(BrainSystems::Execute),
    );
}
