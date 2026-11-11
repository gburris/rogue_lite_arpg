use bevy::ecs::query::QueryData;

use super::{Dash, Motion, Orbit, Swoop, Wander};
use crate::character::brain::CapabilityProfile;

/// Every motion capability component a character may carry.
#[derive(QueryData)]
pub(in crate::character::brain) struct MotionCapabilities {
    dash: Option<&'static Dash>,
    swoop: Option<&'static Swoop>,
    orbit: Option<&'static Orbit>,
    wander: Option<&'static Wander>,
}

impl MotionCapabilitiesItem<'_, '_> {
    /// The profile of the capability that carries out `motion`; `None` when the character lacks
    /// it. Motions steered without a component are always owned.
    pub(in crate::character::brain) fn capability_profile(
        &self,
        motion: Motion,
    ) -> Option<CapabilityProfile> {
        match motion {
            // Their components carry no decision settings, only ownership.
            Motion::Wander => self.wander.map(|_| CapabilityProfile::default()),
            Motion::Orbit(_) => self.orbit.map(|_| CapabilityProfile::default()),
            Motion::Dash => self.dash.map(Dash::capability_profile),
            Motion::Swoop => self.swoop.map(Swoop::capability_profile),
            Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance(_)
            | Motion::Idle => Some(CapabilityProfile::default()),
        }
    }
}
