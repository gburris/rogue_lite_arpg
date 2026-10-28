//! Appliers for the brain's equipment intents.

use bevy::prelude::*;

use super::super::perception::Targeting;
use super::super::{ActionFinished, ActionOutcome, Brain};
use super::Action;
use crate::{
    character::vision::TargetInfo,
    prelude::{EquipmentSlot, EquipmentUsage},
};

/// The last use started in each slot, stamped with its behavior's epoch, so a
/// committed use starts once and its finish names its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(in super::super) struct EquipmentUseEpochs(
    [Option<StartedEquipmentUse>; EquipmentSlot::ALL.len()],
);

/// One started use, awaiting its finish until its item stops being in use.
#[derive(Clone, Copy, Debug)]
struct StartedEquipmentUse {
    epoch: u32,
    awaiting_finish: bool,
}

impl EquipmentUseEpochs {
    fn started_use_mut(&mut self, slot: EquipmentSlot) -> &mut Option<StartedEquipmentUse> {
        &mut self.0[slot as usize]
    }
}

/// Starts and releases the brain's equipment uses. Decisions check range at
/// 10 Hz, so momentum uses are re-gated on the live target every frame; a
/// committed use starts once, unconditionally. Every started use reports
/// [`ActionFinished`] with its epoch once its item stops being in use.
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
            // A start from an earlier frame has flushed, so an item no longer in
            // use has finished, even a use that ended within its first frame.
            if let Some(started) = started_use
                .as_mut()
                .filter(|started| started.awaiting_finish && !in_use)
            {
                started.awaiting_finish = false;
                commands.trigger(ActionFinished {
                    entity,
                    action: Action::Use(slot),
                    epoch: started.epoch,
                    outcome: ActionOutcome::Completed,
                });
            }

            // The same item reach the equipment considerations scored, checked live.
            let reach = equipment
                .equippable_in_slot(entity, slot)
                .and_then(|equippable| equippable.reach);
            let in_range = reach.is_some_and(|reach| {
                target_info.is_some_and(|info| info.distance.is_finite() && info.distance <= reach)
            });
            let live = has_target && sighted && in_range;
            let slot_behavior = brain.held_action(Action::Use(slot));
            let record_start = |epoch| StartedEquipmentUse {
                epoch,
                awaiting_finish: true,
            };

            if let Some(committed) = slot_behavior.filter(|slot_behavior| slot_behavior.committed) {
                if started_use.is_none_or(|started| started.epoch != committed.epoch) {
                    if equipment.try_use(entity, slot).is_ok() {
                        *started_use = Some(record_start(committed.epoch));
                    } else {
                        committed.report_not_started(&mut commands, entity);
                    }
                }
                continue;
            }

            match slot_behavior.filter(|_| live) {
                Some(requested) if !in_use => {
                    // A momentum use waits quietly: a refused `try_use` would
                    // flag mana changed and warn every frame.
                    let began = equipment.can_use(entity, slot).is_ok()
                        && equipment.try_use(entity, slot).is_ok();
                    if began {
                        *started_use = Some(record_start(requested.epoch));
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
