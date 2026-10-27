//! Abilities a brain can ask for besides moving or using equipment. Each one
//! is an opt-in capability component plus an applier that reads the brain's
//! held action and its epoch, re-checks the live world, does the work, and
//! reports [`ActionFinished`](super::super::ActionFinished) for that epoch. A
//! momentum action the live world refuses waits; a committed one reports that
//! it never started.
//! Appliers never change the brain's motion. A defeated character loses its
//! brain, so appliers never see the dead.
//!
//! Capability components implement
//! [`Capability`](super::super::capability::Capability) and are listed once,
//! in [`AbilityCapabilities`], which is how the brain learns what an entity can
//! do without naming each component.

mod bite;
mod breath;
mod heal;

use bevy::{ecs::query::QueryData, prelude::*};

pub use bite::Bite;
pub use breath::Breath;
pub use heal::Heal;
pub(in super::super) use heal::find_carried_potion;

use super::super::capability::{Capabilities, record_capability};
use crate::character::AiSet;

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
    pub(in super::super) fn record_capabilities(&self, capabilities: &mut Capabilities) {
        record_capability(capabilities, self.bite);
        record_capability(capabilities, self.breath);
        record_capability(capabilities, self.heal);
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (heal::apply_heal, breath::apply_breath, bite::apply_bite).in_set(AiSet::Execute),
    );
}
