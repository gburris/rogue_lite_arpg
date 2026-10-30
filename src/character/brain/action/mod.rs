//! What a character does besides moving: the [`Action`] a brain chooses, the
//! equipment applier, and the innate abilities.

mod ability;
mod equipment_use;

use bevy::prelude::*;

pub use self::ability::{Ability, Bite, Breath, Heal};
pub(super) use self::ability::{AbilityCapabilities, find_carried_potion};
pub(super) use self::equipment_use::StartedEquipmentUses;
use super::BrainSystems;
use crate::prelude::*;

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
    app.add_plugins(ability::plugin)
        .add_systems(
            Update,
            equipment_use::apply_equipment_use_requests.in_set(BrainSystems::Execute),
        )
        .add_observer(equipment_use::on_report_equipment_use_finished);
}

/// An equipment or ability use ended. It ends the behavior stamped `behavior_run_id`
/// only while that behavior still runs, so a stale finish clears nothing.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct ActionFinished {
    pub entity: Entity,
    pub action: Action,
    pub behavior_run_id: u32,
    pub outcome: ActionOutcome,
}

/// How an action reported in [`ActionFinished`] ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionOutcome {
    Completed,
    /// A committed use its applier could not start. A continuously scored use waits for
    /// its chance instead and never reports this.
    NotStarted,
}
