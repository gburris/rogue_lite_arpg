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
            // Find the first available slot
            let slot = (0..self.max_capacity as u8)
                .find(|i| !self.items.contains_key(i))
                .ok_or_else(|| "No available slots".to_string())?;

            self.items.insert(slot, item);
            Ok(())
        } else {
            Err("Inventory is full".to_string())
        }
    }

    pub fn remove_item(&mut self, item: Entity) -> Result<(), String> {
        if let Some(slot) = self
            .items
            .iter()
            .find_map(|(&k, &v)| if v == item { Some(k) } else { None })
        {
            self.items.remove(&slot);
            Ok(())
        } else {
            Err("Item not found in inventory".to_string())
        }
    }

    pub fn default_inventory() -> Self {
        Inventory {
            max_capacity: 10,
            items: HashMap::new(),
        }
    }
}
