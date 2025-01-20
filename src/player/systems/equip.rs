use crate::{
    items::EquipmentSlot,
    player::{components::PlayerEquipmentSlots, equip_item, Inventory, Player},
    ui::pause_menu::button_interactions::TryEquipEvent,
};
use bevy::prelude::*;

#[derive(Event)]
pub struct EquipSuccessEvent {
    pub item_entity: Entity,
    pub previous_item: Option<Entity>,
}

pub fn handle_try_equip_event(
    try_equip_trigger: Trigger<TryEquipEvent>,
    mut commands: Commands,
    mut equipment_query: Query<&mut PlayerEquipmentSlots>,
    mut inventory_query: Query<&mut Inventory>,
    slot_query: Query<&EquipmentSlot>,
) {
    if let Ok(mut equipment_slots) = equipment_query.get_single_mut() {
        if let Some(previous_item) = equip_item(
            &mut equipment_slots,
            try_equip_trigger.item_entity,
            &slot_query,
        ) {
            //Case where there was already something in the slot
            if let Ok(mut inventory) = inventory_query.get_single_mut() {
                let _ = inventory.remove_item(try_equip_trigger.item_entity);
                let _ = inventory.add_item(previous_item);
            }
            commands.trigger(EquipSuccessEvent {
                item_entity: try_equip_trigger.item_entity,
                previous_item: Some(previous_item),
            });
        } else {
            if let Ok(mut inventory) = inventory_query.get_single_mut() {
                let _ = inventory.remove_item(try_equip_trigger.item_entity);
            }
            commands.trigger(EquipSuccessEvent {
                item_entity: try_equip_trigger.item_entity,
                previous_item: None,
            });
        }
    }
}

pub fn handle_equip_success_event(
    equip_success_trigger: Trigger<EquipSuccessEvent>,
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
) {
    let Ok(player_entity) = player_query.get_single() else {
        return;
    };

    // If there was a previous item, remove it from the player
    if let Some(previous_item) = equip_success_trigger.previous_item {
        commands
            .entity(player_entity)
            .remove_children(&[previous_item]);
        if let Ok(mut visibility) = visibility_query.get_mut(previous_item) {
            *visibility = Visibility::Hidden;
        }
    }

    // Add the new item as a child of the player
    commands
        .entity(player_entity)
        .add_child(equip_success_trigger.item_entity);
    if let Ok(mut visibility) = visibility_query.get_mut(equip_success_trigger.item_entity) {
        *visibility = Visibility::Visible;
    }
}
