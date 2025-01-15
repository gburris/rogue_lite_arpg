use bevy::prelude::*;
use std::collections::HashMap;

use crate::items::Item;

#[derive(Component, Default, Debug)]
pub struct Inventory {
    pub max_capacity: usize,
    pub items: HashMap<String, Item>, // Stores items by name
}

impl Inventory {
    pub fn add_item(&mut self, item: Item) -> Result<(), &'static str> {
        if self.items.len() < self.max_capacity {
            self.items.insert(item.name.clone(), item);
            Ok(())
        } else {
            Err("Inventory full!")
        }
    }

    pub fn remove_item(&mut self, item_name: &str) -> Option<Item> {
        self.items.remove(item_name)
    }

    pub fn get_item(&self, item_name: &str) -> Option<&Item> {
        self.items.get(item_name)
    }

    pub fn default_inventory() -> Self {
        Inventory {
            max_capacity: 10,
            items: HashMap::new(),
        }
    }
}
