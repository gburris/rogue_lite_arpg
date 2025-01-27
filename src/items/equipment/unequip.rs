use bevy::prelude::*;

use crate::{
    items::{inventory::inventory::Inventory, EquipmentSlot},
    player::Player,
    ui::pause_menu::button_interactions::TryUnequipEvent,
};

use super::equipment::{unequip_item, EquipmentSlots};

#[derive(Event)]

pub struct UnequipSuccessEvent {
    pub item_entity: Entity,
}

pub fn handle_try_unequip_event(
    try_equip_trigger: Trigger<TryUnequipEvent>,
    mut commands: Commands,
    mut equipment_query: Query<&mut EquipmentSlots>,
    mut inventory_query: Query<&mut Inventory>,
    slot_query: Query<&EquipmentSlot>,
) {
    if let Ok(mut equipment_slots) = equipment_query.get_single_mut() {
        if let Ok(mut inventory) = inventory_query.get_single_mut() {
            let did_add_item = inventory.add_item(try_equip_trigger.item_entity);
            if did_add_item.is_ok() {
                unequip_item(
                    &mut equipment_slots,
                    try_equip_trigger.item_entity,
                    &slot_query,
                );
                commands.trigger(UnequipSuccessEvent {
                    item_entity: try_equip_trigger.item_entity,
                });
            } else {
                warn!("Inventory was full! Cannot unequip weapon");
            }
        }
    }
}

pub fn handle_unequip_success_event(
    unequip_success_trigger: Trigger<UnequipSuccessEvent>,
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
) {
    let player_entity = player_query.get_single();

    // If there was a previous item, remove it from the player
    let previous_item = unequip_success_trigger.item_entity;
    commands
        .entity(player_entity.unwrap())
        .remove_children(&[previous_item]);
    if let Ok(mut visibility) = visibility_query.get_mut(previous_item) {
        *visibility = Visibility::Hidden;
    }
}
