//! Appliers for the brain's equipment intents.

use bevy::prelude::*;

use super::{Action, ActionIntent, Brain};
use crate::{
    character::vision::{TargetInfo, Targeting},
    prelude::{EquipmentSlot, EquipmentUsage},
};

/// Re-gates stale intent on the live target state. Decisions price range at
/// 10 Hz but this runs every frame, so a target that drifted
/// out of reach between ticks must not keep a stale `try_use` alive.
pub(super) fn apply_slot_requests(
    mut equipment: EquipmentUsage,
    requests: Query<(
        Entity,
        &Brain,
        &ActionIntent,
        Has<Targeting>,
        Option<&TargetInfo>,
    )>,
) {
    for (entity, brain, action_intent, has_target, target_info) in &requests {
        // The same reach the equipment considerations scored, checked live.
        let in_range = brain.definition.equipment_reach.is_some_and(|reach| {
            target_info.is_some_and(|info| info.distance.is_finite() && info.distance <= reach)
        });
        for slot in EquipmentSlot::ALL {
            let sighted = target_info.is_some_and(|info| info.line_of_sight);
            let requested =
                has_target && sighted && in_range && action_intent.get(Action::Use(slot));
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
