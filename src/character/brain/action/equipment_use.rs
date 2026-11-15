//! Appliers for the brain's equipment intents.

use bevy::prelude::*;

use super::{Action, ActionFinished, ActionOutcome};
use crate::character::brain::Brain;
use crate::prelude::*;

/// The last use started in each slot, stamped with its behavior's run id, so a
/// committed use starts once and its item's finish names its behavior.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(in crate::character::brain) struct StartedEquipmentUses(
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

    fn find_started_uses_of_item(
        &self,
        item: Entity,
    ) -> impl Iterator<Item = (EquipmentSlot, StartedEquipmentUse)> + '_ {
        EquipmentSlot::ALL.into_iter().filter_map(move |slot| {
            self.0[slot as usize]
                .filter(|started| started.item == item)
                .map(|started| (slot, started))
        })
    }
}

/// Reports [`ActionFinished`] with the run id of the use whose item finished.
pub(super) fn on_report_equipment_use_finished(
    finished: On<EquipmentUseFinished>,
    mut commands: Commands,
    started_equipment_uses: Query<&StartedEquipmentUses>,
) {
    let Ok(started_equipment_uses) = started_equipment_uses.get(finished.holder) else {
        return;
    };
    for (slot, started) in started_equipment_uses.find_started_uses_of_item(finished.item) {
        commands.trigger(ActionFinished {
            entity: finished.holder,
            action: Action::Use(slot),
            behavior_run_id: started.behavior_run_id,
            outcome: ActionOutcome::Completed,
        });
    }
}

/// Starts and releases the brain's equipment uses. Considerations already judged the target, so
/// the item itself decides only whether it can start; see [`EquipmentUsage::try_use`]. A committed
/// use starts once; a continuously scored one again whenever its item allows. A held use is
/// released once no behavior holds it.
pub(super) fn apply_equipment_use_requests(
    mut commands: Commands,
    mut equipment: EquipmentUsage,
    mut requests: Query<(Entity, &Brain, &mut StartedEquipmentUses)>,
) {
    for (entity, brain, mut started_equipment_uses) in &mut requests {
        for slot in EquipmentSlot::ALL {
            let in_use = equipment.is_using(entity, slot);
            let Some(requested) = brain.held_action(Action::Use(slot)) else {
                let is_held_use = equipment
                    .equippable_in_slot(entity, slot)
                    .is_some_and(|equippable| equippable.use_style == EquipmentUseStyle::Held);
                if in_use && is_held_use {
                    equipment.stop_using(entity, slot);
                }
                continue;
            };
            let started_use = started_equipment_uses.started_use_mut(slot);
            let already_started = if requested.committed {
                started_use
                    .is_some_and(|started| started.behavior_run_id == requested.behavior_run_id)
            } else {
                in_use
            };
            if already_started {
                continue;
            }
            match equipment.try_use(entity, slot) {
                Ok(item) => {
                    *started_use = Some(StartedEquipmentUse {
                        behavior_run_id: requested.behavior_run_id,
                        item,
                    });
                }
                Err(_) => requested.report_not_started(&mut commands, entity),
            }
        }
    }
}
