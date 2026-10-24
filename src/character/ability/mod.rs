//! Abilities a brain can ask for besides moving or using equipment. Each one
//! is an opt-in capability component plus an applier that reads the brain's
//! [`AbilityIntent`](super::brain::AbilityIntent), re-checks the live world,
//! and does the work. Appliers never start a maneuver or change a mode.
//!
//! Capability components implement [`AbilityCapability`] and are listed once,
//! in [`AbilityCapabilities`], which is how the brain's senses learn what an
//! entity can do without naming each component.

mod bite;
mod breath;
mod heal;

use bevy::{ecs::query::QueryData, prelude::*};

pub use bite::Bite;
pub use breath::Breath;
pub use heal::Heal;

use super::AiSet;
use super::brain::{Ability, AbilityTuning};

/// A capability component that carries out one [`Ability`].
pub trait AbilityCapability {
    fn ability(&self) -> Ability;
    /// What the decision tick needs to know to score this ability.
    fn tuning(&self) -> AbilityTuning;
}

/// Every ability capability an entity may carry. The one place to list a new one.
#[derive(QueryData)]
pub struct AbilityCapabilities {
    bite: Option<&'static Bite>,
    breath: Option<&'static Breath>,
    heal: Option<&'static Heal>,
}

impl AbilityCapabilitiesItem<'_, '_> {
    /// Tuning of every ability this entity can carry out, by channel index.
    pub fn tuning(&self) -> [Option<AbilityTuning>; Ability::ALL.len()] {
        let capabilities: [Option<&dyn AbilityCapability>; 3] = [
            self.bite.map(|capability| capability as &dyn AbilityCapability),
            self.breath.map(|capability| capability as &dyn AbilityCapability),
            self.heal.map(|capability| capability as &dyn AbilityCapability),
        ];
        let mut tuning = [None; Ability::ALL.len()];
        for capability in capabilities.into_iter().flatten() {
            tuning[capability.ability() as usize] = Some(capability.tuning());
        }
        tuning
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (heal::apply_heal, breath::apply_breath, bite::apply_bite).in_set(AiSet::Execute),
    );
}
