use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component, Default, Debug)]
pub struct Inventory {
    pub max_capacity: usize,
    pub items: VecDeque<Entity>,

    /// Equipment slots
    pub mainhand: Option<Entity>,
    pub offhand: Option<Entity>,

    /// If you want to open this inventory in a UI    
    pub display_case: Option<Entity>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            max_capacity: 10,
            items: VecDeque::new(),
            ..default()
        }
    }
}

impl Inventory {
    pub fn add_item(&mut self, item: Entity) -> Result<(), String> {
        if self.items.len() < self.max_capacity {
            self.items.push_back(item);
            Ok(())
        } else {
            Err("Inventory is full".to_string())
        }
    }

    pub fn remove_item_by_value(&mut self, item: Entity) -> Result<Entity, String> {
        // Search for item by comparing values (entities) and then remove by index
        if let Some(item_index) = self.items.iter().position(|&e| e == item) {
            self.items
                .remove(item_index)
                .ok_or("Index was out of bounds".to_string())
        } else {
            Err("Item not found in inventory".to_string())
        }
    }

    pub fn remove_item(&mut self, index: usize) -> Result<Entity, String> {
        self.items
            .remove(index)
            .ok_or("Index was out of bounds".to_string())
    }

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
            EquipmentSlot::Offhand => &mut self.head,
        };

        *slot_ref = None;
    }
}
