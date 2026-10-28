//! Appliers for the brain's equipment intents.

use bevy::prelude::*;

use super::super::perception::Targeting;
use super::super::{ActionFinished, ActionOutcome, Brain};
use super::Action;
use crate::{
    character::vision::TargetInfo,
    prelude::{EquipmentSlot, EquipmentUsage, EquipmentUseFinished},
};

/// The last use started in each slot, stamped with its behavior's epoch, so a
/// committed use starts once and its item's finish names its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(in super::super) struct EquipmentUseEpochs(
    [Option<StartedEquipmentUse>; EquipmentSlot::ALL.len()],
);

#[derive(Clone, Copy, Debug)]
struct StartedEquipmentUse {
    epoch: u32,
    item: Entity,
}

impl EquipmentUseEpochs {
    fn started_use_mut(&mut self, slot: EquipmentSlot) -> &mut Option<StartedEquipmentUse> {
        &mut self.0[slot as usize]
    }
}

/// Reports [`ActionFinished`] with the epoch of the use whose item finished.
pub(super) fn report_equipment_use_finished(
    finished: On<EquipmentUseFinished>,
    mut commands: Commands,
    use_epochs: Query<&EquipmentUseEpochs>,
) {
    let Ok(use_epochs) = use_epochs.get(finished.holder) else {
        return;
    };
    for slot in EquipmentSlot::ALL {
        if let Some(started) =
            use_epochs.0[slot as usize].filter(|started| started.item == finished.item)
        {
            commands.trigger(ActionFinished {
                entity: finished.holder,
                action: Action::Use(slot),
                epoch: started.epoch,
                outcome: ActionOutcome::Completed,
            });
        }
    }
}

/// Starts and releases the brain's equipment uses. Decisions check range at
/// 10 Hz, so momentum uses are re-gated on the live target every frame; a
/// committed use starts once, unconditionally.
pub(super) fn apply_equipment_use_requests(
    mut commands: Commands,
    mut equipment: EquipmentUsage,
    mut requests: Query<(
        Entity,
        &Brain,
        &mut EquipmentUseEpochs,
        Has<Targeting>,
        Option<&TargetInfo>,
    )>,
) {
    for (entity, brain, mut use_epochs, has_target, target_info) in &mut requests {
        let sighted = target_info.is_some_and(|info| info.line_of_sight);
        for slot in EquipmentSlot::ALL {
            let in_use = equipment.is_using(entity, slot);
            let started_use = use_epochs.started_use_mut(slot);

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
                if started_use.is_none_or(|started| started.epoch != committed.epoch) {
                    match equipment.try_use(entity, slot) {
                        Ok(item) => {
                            *started_use = Some(StartedEquipmentUse {
                                epoch: committed.epoch,
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
                            epoch: requested.epoch,
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
