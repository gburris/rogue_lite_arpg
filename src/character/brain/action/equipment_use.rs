//! Appliers for the brain's equipment intents.

use bevy::prelude::*;

use super::super::perception::Targeting;
use super::super::{ActionFinished, ActionOutcome, Brain};
use super::Action;
use crate::character::vision::TargetInfo;
use crate::prelude::{EquipmentSlot, EquipmentUsage, EquipmentUseFinished};

/// The last use started in each slot, stamped with its behavior's run id, so a
/// committed use starts once and its item's finish names its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(in super::super) struct StartedEquipmentUses(
    [Option<StartedEquipmentUse>; EquipmentSlot::ALL.len()],
);

#[derive(Clone, Copy, Debug)]
struct StartedEquipmentUse {
    behavior_run_id: u32,
    item: Entity,
}

impl StartedEquipmentUses {
    fn started_use_mut(&mut self, slot: EquipmentSlot) -> &mut Option<StartedEquipmentUse> {
        &mut self.0[slot as usize]
    }
}

/// Reports [`ActionFinished`] with the run id of the use whose item finished.
pub(super) fn report_equipment_use_finished(
    finished: On<EquipmentUseFinished>,
    mut commands: Commands,
    started_equipment_uses: Query<&StartedEquipmentUses>,
) {
    let Ok(started_equipment_uses) = started_equipment_uses.get(finished.holder) else {
        return;
    };
    for slot in EquipmentSlot::ALL {
        if let Some(started) =
            started_equipment_uses.0[slot as usize].filter(|started| started.item == finished.item)
        {
            commands.trigger(ActionFinished {
                entity: finished.holder,
                action: Action::Use(slot),
                behavior_run_id: started.behavior_run_id,
                outcome: ActionOutcome::Completed,
            });
        }
    }
}

/// Starts and releases the brain's equipment uses. Decisions check range at
/// 10 Hz, so continuously scored uses are re-gated on the live target every frame; a
/// committed use starts once, unconditionally.
pub(super) fn apply_equipment_use_requests(
    mut commands: Commands,
    mut equipment: EquipmentUsage,
    mut requests: Query<(
        Entity,
        &Brain,
        &mut StartedEquipmentUses,
        Has<Targeting>,
        Option<&TargetInfo>,
    )>,
) {
    for (entity, brain, mut started_equipment_uses, has_target, target_info) in &mut requests {
        let sighted = target_info.is_some_and(|info| info.line_of_sight);
        for slot in EquipmentSlot::ALL {
            let in_use = equipment.is_using(entity, slot);
            let started_use = started_equipment_uses.started_use_mut(slot);

            // The same item reach the equipment considerations scored, checked live.
            let reach = equipment
                .equippable_in_slot(entity, slot)
                .and_then(|equippable| equippable.reach);
            let in_range = reach.is_some_and(|reach| {
                target_info.is_some_and(|info| info.distance.is_finite() && info.distance <= reach)
            });
            let live = has_target && sighted && in_range;
            let slot_behavior = brain.held_action(Action::Use(slot));

            if let Some(committed) = slot_behavior.filter(|slot_behavior| slot_behavior.committed) {
                if started_use
                    .is_none_or(|started| started.behavior_run_id != committed.behavior_run_id)
                {
                    match equipment.try_use(entity, slot) {
                        Ok(item) => {
                            *started_use = Some(StartedEquipmentUse {
                                behavior_run_id: committed.behavior_run_id,
                                item,
                            });
                        }
                        Err(_) => committed.report_not_started(&mut commands, entity),
                    }
                }
                continue;
            }

            match slot_behavior.filter(|_| live) {
                Some(requested) if !in_use => {
                    if let Ok(item) = equipment.try_use(entity, slot) {
                        *started_use = Some(StartedEquipmentUse {
                            behavior_run_id: requested.behavior_run_id,
                            item,
                        });
                    }
                }
                None if in_use && !slot.is_instant_use() => {
                    equipment.stop_using(entity, slot);
                }
                _ => {}
            }
        }
    }
}
