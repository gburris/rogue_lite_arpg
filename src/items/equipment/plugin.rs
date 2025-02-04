use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{
    equip::{
        handle_try_equip_event, handle_try_unequip_event, handle_unequip_success_event,
        on_equipment_slot_equip, tick_equippable_use_rate,
    },
    equipment_transform::update_equipment_transforms,
};

pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                update_equipment_transforms,
                on_equipment_slot_equip,
                tick_equippable_use_rate,
            ))
                .in_set(InGameSet::Simulation),
        )
        .add_observer(handle_try_equip_event)
        .add_observer(handle_try_unequip_event)
        .add_observer(handle_unequip_success_event);
    }
}
