//! Appliers for the brain's equipment intents.

use bevy::prelude::*;

use super::{Brain, SlotIntent};
use crate::{
    character::vision::{TargetInfo, Targeting},
    prelude::{EquipmentSlot, EquipmentUsage, Health},
};

/// Re-gates stale intent on the live target state; skips the dead. Decisions
/// price range at 10 Hz but this runs every frame, so a target that drifted
/// out of reach between ticks must not keep a stale `try_use` alive.
pub(super) fn apply_slot_requests(
    mut equipment: EquipmentUsage,
    requests: Query<(
        Entity,
        &Brain,
        &SlotIntent,
        Has<Targeting>,
        Option<&TargetInfo>,
        Option<&Health>,
    )>,
) {
    for (entity, brain, slot_intent, has_target, target_info, health) in &requests {
        if health.is_some_and(|health| health.hp <= 0.0) {
            continue;
        }
        // Live range check against the definition's reach, which mirrors the
        // slot considerations. No reach keeps the decision-tick intent.
        let in_range = brain.definition.slot_reach.is_none_or(|reach| {
            target_info.is_some_and(|info| info.distance.is_finite() && info.distance <= reach)
        });
        for slot in EquipmentSlot::ALL {
            let sighted = target_info.is_some_and(|info| info.line_of_sight);
            let requested = has_target && sighted && in_range && slot_intent.get(slot);
            let in_use = equipment.is_using(entity, slot);

            if requested && !in_use {
                if equipment.can_use(entity, slot).is_ok() {
                    let _ = equipment.try_use(entity, slot);
                }
            } else if !requested && in_use && !slot.is_instant_use() {
                equipment.stop_using(entity, slot);
            }
        }
    }
}
