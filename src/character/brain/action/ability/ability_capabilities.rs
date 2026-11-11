use bevy::ecs::query::QueryData;

use super::{Ability, Bite, Breath, Heal};
use crate::character::brain::CapabilityProfile;

/// Every ability capability component a character may carry.
#[derive(QueryData)]
pub(in crate::character::brain) struct AbilityCapabilities {
    bite: Option<&'static Bite>,
    breath: Option<&'static Breath>,
    heal: Option<&'static Heal>,
}

impl AbilityCapabilitiesItem<'_, '_> {
    /// The profile of the component that carries out `ability`; `None` when the character
    /// lacks it.
    pub(in crate::character::brain) fn capability_profile(
        &self,
        ability: Ability,
    ) -> Option<CapabilityProfile> {
        match ability {
            Ability::Heal => self.heal.map(|heal| heal.capability_profile()),
            Ability::Breathe => self.breath.map(Breath::capability_profile),
            Ability::Bite => self.bite.map(Bite::capability_profile),
        }
    }
}
