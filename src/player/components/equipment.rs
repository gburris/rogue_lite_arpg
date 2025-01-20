use crate::items::EquipmentSlot; //Move this here
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerEquipmentSlots {
    pub mainhand: Option<Entity>,
    pub head: Option<Entity>,
}

impl Default for PlayerEquipmentSlots {
    fn default() -> Self {
        PlayerEquipmentSlots {
            mainhand: None,
            head: None,
        }
    }
}

impl PlayerEquipmentSlots {
    fn add_equipment(&mut self, slot: EquipmentSlot, new_item: Entity) -> Option<Entity> {
        let slot_ref = match slot {
            EquipmentSlot::Mainhand => &mut self.mainhand,
            EquipmentSlot::Helmet => &mut self.head,
        };

        let previous = slot_ref.take();

        *slot_ref = Some(new_item);

        previous
    }

    fn remove_equipment(&mut self, slot: EquipmentSlot) {
        let slot_ref = match slot {
            EquipmentSlot::Mainhand => &mut self.mainhand,
            EquipmentSlot::Helmet => &mut self.head,
        };

        *slot_ref = None;
    }
}

//Public API below, just equip or remove
pub fn equip_item(
    equipment_slots: &mut PlayerEquipmentSlots,
    new_item: Entity,
    slot_query: &Query<&EquipmentSlot>,
) -> Option<Entity> {
    // Get the equipment slot from the item
    if let Ok(slot) = slot_query.get(new_item) {
        // Add the new item and get back any previous item that was equipped
        equipment_slots.add_equipment(slot.clone(), new_item)
    } else {
        None
    }
}

pub fn unequip_item(
    equipment_slots: &mut PlayerEquipmentSlots,
    new_item: Entity,
    slot_query: &Query<&EquipmentSlot>,
) {
    // Get the equipment slot from the item
    if let Ok(slot) = slot_query.get(new_item) {
        // Add the new item and get back any previous item that was equipped
        equipment_slots.remove_equipment(slot.clone());
    }
}
