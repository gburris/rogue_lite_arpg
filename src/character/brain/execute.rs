//! Appliers for the brain's equipment intents.

use bevy::prelude::*;

use super::{Action, ActionAborted, Brain};
use crate::{
    character::vision::{TargetInfo, Targeting},
    prelude::{EquipmentSlot, EquipmentUsage},
};

/// The locked action run the applier already started, so a lock begins a use
/// once and is never re-gated afterwards.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) struct StartedLockedEquipmentUse(Option<u32>);

/// Re-gates the brain's held action on the live target state. Decisions price
/// range at 10 Hz but this runs every frame, so a target that drifted out of
/// reach between ticks must not keep a stale `try_use` alive. A locked use is
/// gated once, when it starts: a failed start aborts the lock, and a started
/// one runs to its finish event.
pub(super) fn apply_equipment_use_requests(
    mut commands: Commands,
    mut equipment: EquipmentUsage,
    mut requests: Query<(
        Entity,
        &Brain,
        &mut StartedLockedEquipmentUse,
        Has<Targeting>,
        Option<&TargetInfo>,
    )>,
) {
    for (entity, brain, mut started, has_target, target_info) in &mut requests {
        let sighted = target_info.is_some_and(|info| info.line_of_sight);
        let locked = brain.active_action();
        for slot in EquipmentSlot::ALL {
            // The same item reach the equipment considerations scored, checked live.
            let reach = equipment
                .equippable_in_slot(entity, slot)
                .and_then(|equippable| equippable.reach);
            let in_range = reach.is_some_and(|reach| {
                target_info.is_some_and(|info| info.distance.is_finite() && info.distance <= reach)
            });
            let live = has_target && sighted && in_range;
            if let Some(active) = locked.filter(|active| active.action == Action::Use(slot)) {
                if started.0 != Some(active.epoch) {
                    started.0 = Some(active.epoch);
                    let began = live
                        && equipment.can_use(entity, slot).is_ok()
                        && equipment.try_use(entity, slot).is_ok();
                    if !began {
                        commands.trigger(ActionAborted { entity });
                    }
                }
                continue;
            }

            let requested = live && brain.action() == Some(Action::Use(slot));
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
