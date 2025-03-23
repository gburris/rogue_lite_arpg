use bevy::prelude::*;

use crate::{
    items::equipment::*,
    labels::sets::{InGameSet, MainSet},
};

pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Always run this system InGame and InMenu so weapon transforms update as inventory is interacted with
                equipment_transform::update_equipment_transforms.in_set(MainSet::Shared),
                use_equipped::tick_equippable_use_rate.in_set(InGameSet::Simulation),
            ),
        )
        .add_observer(equip::on_item_equipped)
        .add_observer(unequip::on_item_unequipped);
    }
}
