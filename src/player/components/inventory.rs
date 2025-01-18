use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Default, Debug)]
pub struct Inventory {
    pub max_capacity: usize,
    pub items: HashMap<u8, Entity>,
}

impl Inventory {
    pub fn add_item(&mut self, item: Entity) -> Result<(), String> {
        if self.items.len() < self.max_capacity {
            let slot = self.items.len() as u8;
            self.items.insert(slot, item);
            Ok(())
        } else {
            Err("Inventory is full".to_string())
        }
    }

    pub fn default_inventory() -> Self {
        Inventory {
            max_capacity: 10,
            items: HashMap::new(),
        }
    }
}
