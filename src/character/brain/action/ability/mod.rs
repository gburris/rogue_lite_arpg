//! Innate abilities. Each is an opt-in capability component, listed in
//! [`AbilityCapabilities`], plus an applier that re-checks the live world for
//! the brain's held action and reports
//! [`ActionFinished`](crate::character::brain::ActionFinished). Appliers never change the
//! motion, and a defeated character loses its brain, so they never see the dead.

mod bite;
mod breath;
mod heal;

use bevy::ecs::query::QueryData;
use bevy::prelude::*;
pub use bite::Bite;
pub use breath::Breath;
pub use heal::Heal;
pub(in crate::character::brain) use heal::find_carried_potion;

use crate::character::AiSet;
use crate::character::brain::capability::{CapabilityProfiles, record_capability_profile};

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

/// Every ability capability an entity may carry. The one place to list a new one.
#[derive(QueryData)]
pub struct AbilityCapabilities {
    bite: Option<&'static Bite>,
    breath: Option<&'static Breath>,
    heal: Option<&'static Heal>,
}

impl AbilityCapabilitiesItem<'_, '_> {
    pub(in crate::character::brain) fn record_capability_profiles(
        &self,
        capability_profiles: &mut CapabilityProfiles,
    ) {
        record_capability_profile(capability_profiles, self.bite);
        record_capability_profile(capability_profiles, self.breath);
        record_capability_profile(capability_profiles, self.heal);
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (heal::apply_heal, breath::apply_breath, bite::apply_bite).in_set(AiSet::Execute),
    );
}
