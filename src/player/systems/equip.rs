use crate::{
    items::{EquipmentSlot, EquipmentSprite},
    player::{components::PlayerEquipmentSlots, Inventory, Player},
    ui::button_interactions::EquipEvent,
};
use bevy::prelude::*;

pub fn handle_equip_event(
    trigger: Trigger<EquipEvent>,
    mut commands: Commands,
    mut equipment_query: Query<&mut PlayerEquipmentSlots>,
    mut source_query: Query<(Entity, &mut Inventory)>, // Ensure this is mutable
    equipment_slot_query: Query<&EquipmentSlot>,
    player_query: Query<Entity, With<Player>>,
    mut equipment_sprite_query: Query<&mut EquipmentSprite>, // Make it mutable to modify it
) {
    let Ok(mut player_equipment) = equipment_query.get_single_mut() else {
        warn!("No player equipment found");
        return;
    };

    let Ok(player_entity) = player_query.get_single() else {
        warn!("Player entity not found");
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

    let Ok(equipment_sprite) = equipment_sprite_query.get_mut(trigger.item_entity) else {
        warn!("No equipment sprite found for entity");
        return;
    };

    // Apply the equipment sprite's offset and scale to the sprite
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
            commands
                .entity(player_entity)
                .add_child(trigger.item_entity);

            // Apply the equipment sprite offset and scale to the sprite
            commands.entity(trigger.item_entity).insert(Transform {
                translation: equipment_sprite.offset,
                scale: equipment_sprite.scale,
                rotation: equipment_sprite.rotation,
                ..Default::default()
            });
        }

        if *slot_type == EquipmentSlot::Helmet {
            player_equipment.head = Some(trigger.item_entity);
            commands
                .entity(player_entity)
                .add_child(trigger.item_entity);

            // Apply the equipment sprite offset and scale to the sprite
            commands.entity(trigger.item_entity).insert(Transform {
                translation: equipment_sprite.offset,
                scale: equipment_sprite.scale,
                rotation: equipment_sprite.rotation,
                ..Default::default()
            });
        }
    }
}
