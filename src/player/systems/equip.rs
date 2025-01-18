use crate::{
    items::EquipmentSlot,
    player::{components::PlayerEquipmentSlots, Inventory},
    ui::button_interactions::EquipEvent,
};
use bevy::prelude::*;

pub fn handle_equip_event(
    trigger: Trigger<EquipEvent>,
    mut equipment_query: Query<&mut PlayerEquipmentSlots>,
    mut source_query: Query<(Entity, &mut Inventory)>, // Ensure this is mutable
    equipment_slot_query: Query<&EquipmentSlot>,
) {
    let Ok(mut player_equipment) = equipment_query.get_single_mut() else {
        warn!("No player equipment found");
        return;
    };

    let Ok((_player_entity, mut inventory)) = source_query.get_single_mut() else {
        // Use get_single_mut
        warn!("No player inventory found");
        return;
    };

    // Use item_entity instead of trigger.entity()
    let Ok(slot_type) = equipment_slot_query.get(trigger.item_entity) else {
        warn!("No equipment slot type found for entity");
        return;
    };

    warn!("trying to equip the equipable entity 1");
    // Find and remove the item from inventory
    if let Some(slot) = inventory
        .items
        .iter()
        .find(|(_, &entity)| entity == trigger.item_entity)
        .map(|(&slot, _)| slot)
    {
        // Perform the removal separately to avoid borrow conflicts
        inventory.items.remove(&slot);

        // Equip the new item
        if *slot_type == EquipmentSlot::Mainhand {
            player_equipment.mainhand = Some(trigger.item_entity);
        }
    }
}
