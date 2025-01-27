use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{
    equip::{handle_equip_success_event, handle_try_equip_event, tick_equippable_use_rate},
    equipment_transform::update_equipment_transforms,
    unequip::{handle_try_unequip_event, handle_unequip_success_event},
};

pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                update_equipment_transforms, //GENERIC SYSTEM TO UPDATE ALL EQUIPMENT, MOVE TO EQUIPMENT PLUGIN
                tick_equippable_use_rate,
            ))
                .in_set(InGameSet::Simulation),
        )
        //TODO: Move to equipment plugin
        .add_observer(handle_try_equip_event)
        .add_observer(handle_equip_success_event)
        .add_observer(handle_try_unequip_event)
        .add_observer(handle_unequip_success_event);
    }
}
