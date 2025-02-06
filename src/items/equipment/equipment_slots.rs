use bevy::prelude::*;

/**
 * Goes on the equipment marking where it should be equipped
 */
#[derive(Clone)]
pub enum EquipmentSlot {
    Mainhand,
    Helmet,
}

/**
 * Goes on the holder marking what is has equipped
 */
#[derive(Component, Default)]
pub struct EquipmentSlots {
    pub mainhand: Option<Entity>,
    pub head: Option<Entity>,
}

impl EquipmentSlots {
    /// Equip the new_item in the specified slot
    pub fn equip(&mut self, new_item: Entity, slot: &EquipmentSlot) -> Option<Entity> {
        let slot_ref = match slot {
            EquipmentSlot::Mainhand => &mut self.mainhand,
            EquipmentSlot::Helmet => &mut self.head,
        };

        let previous = slot_ref.take();

        *slot_ref = Some(new_item);

        previous
    }

    /// Remove the existing item from the specified slot, if it exists
    pub fn unequip(&mut self, slot: &EquipmentSlot) {
        let slot_ref = match slot {
            EquipmentSlot::Mainhand => &mut self.mainhand,
            EquipmentSlot::Helmet => &mut self.head,
        };

        *slot_ref = None;
    }
}
