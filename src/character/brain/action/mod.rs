//! What a character does besides moving: the [`Action`] a brain chooses, the
//! equipment applier, and the innate abilities.

mod ability;
mod equipment_use;

use bevy::prelude::*;

pub use ability::{Ability, Bite, Breath, Heal};
pub(super) use ability::{AbilityCapabilities, find_carried_potion};
pub(super) use equipment_use::EquipmentUseEpochs;

use crate::{character::AiSet, items::prelude::EquipmentSlot};

/// The one non-movement thing a character does at a time: use equipment, use an
/// ability, or deliberately do nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Use the item equipped in a slot.
    Use(EquipmentSlot),
    Ability(Ability),
    /// Chose to do nothing, unlike an empty action slot where nothing scored.
    /// Always available; no applier carries it out.
    Idle,
}

impl Action {
    /// Number of distinct actions, for per-action arrays.
    pub const COUNT: usize = EquipmentSlot::ALL.len() + Ability::ALL.len() + 1;

    /// Position of this action in per-action arrays.
    pub const fn index(self) -> usize {
        match self {
            Self::Use(slot) => slot as usize,
            Self::Ability(ability) => EquipmentSlot::ALL.len() + ability as usize,
            Self::Idle => EquipmentSlot::ALL.len() + Ability::ALL.len(),
        }
    }

    /// Whether this action ends on its own and reports it, so a committed
    /// behavior over it can finish. A held use ends only when released.
    pub fn reports_finish(self) -> bool {
        match self {
            Self::Use(slot) => slot.is_instant_use(),
            Self::Ability(_) => true,
            Self::Idle => false,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(ability::plugin).add_systems(
        Update,
        equipment_use::apply_equipment_use_requests.in_set(AiSet::Execute),
    );
}
