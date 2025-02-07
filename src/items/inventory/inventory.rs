use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component, Default, Debug)]
pub struct Inventory {
    pub max_capacity: usize,
    pub items: VecDeque<Entity>,
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

    pub fn default_inventory() -> Self {
        Inventory {
            max_capacity: 10,
            items: VecDeque::new(),
        }
    }
}
