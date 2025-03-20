use bevy::prelude::*;
use std::collections::VecDeque;

use crate::items::equipment::EquipmentSlot;

#[derive(Component)]
pub struct Inventory {
    pub max_capacity: usize,
    pub items: VecDeque<Entity>,
    pub coins: u32,
    mainhand_index: Option<usize>,
    offhand_index: Option<usize>,

    /// If you want to open this inventory in a UI    
    pub display_case: Option<Entity>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            max_capacity: 10,
            items: VecDeque::new(),
            coins: 0,
            mainhand_index: None,
            offhand_index: None,
            display_case: None,
        }
    }
}

impl Inventory {
    pub const ALL_SLOTS: [EquipmentSlot; 2] = [EquipmentSlot::Mainhand, EquipmentSlot::Offhand];

    pub fn builder() -> InventoryBuilder {
        InventoryBuilder::default()
    }

    /// Adds an item to the inventory if there's space
    /// Returns the index of the added item
    pub fn add_item(&mut self, item: Entity) -> Result<usize, String> {
        if self.items.len() < self.max_capacity {
            self.items.push_back(item);
            Ok(self.items.len() - 1)
        } else {
            Err("Inventory is full".to_string())
        }
    }

    pub fn remove_item(&mut self, item: Entity) -> Result<Entity, String> {
        // Search for item by comparing values (entities) and then remove by index
        if let Some(item_index) = self.items.iter().position(|&e| e == item) {
            self.remove_item_by_index(item_index)
        } else {
            Err("Item not found in inventory".to_string())
        }
    }

    /// Equip the new_item in the specified slot
    ///
    /// This method expects the equipment to already exist in the inventory.
    /// But if it does not it will still attempt to add it. If that fails, you messed up calling this function.
    ///
    /// Performance: Equipping linearly searches inventory to find item by comparing entities
    /// Consider: Adding a reverse mapping entity_to_index: HashMap<Entity, usize> if we care about making this O(1)
    pub fn equip(&mut self, item: Entity, slot: EquipmentSlot) {
        let index = self.find_item_by_entity(item);

        if index.is_none() {
            let index = self.add_item(item).expect("Why did you try to equip an item outside the inventory while said inventory was full you dummy");
            *self.get_equipped_slot_mut(slot) = Some(index);
        } else {
            *self.get_equipped_slot_mut(slot) = index;
        }
    }

    /// Sets the specified equipment slot to None in inventory
    pub fn unequip(&mut self, item_entity: Entity, slot: EquipmentSlot) {
        if let Some(index) = self.get_equipped_slot(slot) {
            // Only unequip if entity matches
            if item_entity == self.items[index] {
                self.get_equipped_slot_mut(slot).take();
            }
        }
    }

    pub fn get_equipped(&self, slot: EquipmentSlot) -> Option<Entity> {
        self.get_equipped_slot(slot)
            .and_then(|i| self.items.get(i).cloned())
    }

    pub fn add_coins(&mut self, amount: u32) {
        self.coins += amount;
    }

    pub fn remove_coins(&mut self, amount: u32) -> Result<u32, String> {
        if self.coins >= amount {
            self.coins -= amount;
            Ok(self.coins)
        } else {
            Err("Not enough coins!".to_string())
        }
    }

    fn remove_item_by_index(&mut self, index_to_remove: usize) -> Result<Entity, String> {
        // all equipment indicies shift
        for slot in Inventory::ALL_SLOTS {
            self.adjust_slot_index(slot, index_to_remove);
        }

        self.items
            .remove(index_to_remove)
            .ok_or("Index was out of bounds".to_string())
    }

    fn adjust_slot_index(&mut self, slot: EquipmentSlot, index_to_remove: usize) {
        if let Some(slot_index) = self.get_equipped_slot(slot) {
            *self.get_equipped_slot_mut(slot) = match index_to_remove.cmp(&slot_index) {
                std::cmp::Ordering::Less => Some(slot_index - 1),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(slot_index),
            };
        }
    }

    fn find_item_by_entity(&self, item: Entity) -> Option<usize> {
        self.items.iter().position(|&e| e == item)
    }

    fn get_equipped_slot_mut(&mut self, slot: EquipmentSlot) -> &mut Option<usize> {
        match slot {
            EquipmentSlot::Mainhand => &mut self.mainhand_index,
            EquipmentSlot::Offhand => &mut self.offhand_index,
        }
    }

    fn get_equipped_slot(&self, slot: EquipmentSlot) -> Option<usize> {
        match slot {
            EquipmentSlot::Mainhand => self.mainhand_index,
            EquipmentSlot::Offhand => self.offhand_index,
        }
    }
}

pub struct InventoryBuilder {
    max_capacity: usize,
    items: Vec<Entity>,
    coins: u32,
    display_case: Option<Entity>,
}

impl Default for InventoryBuilder {
    fn default() -> Self {
        Self {
            max_capacity: 10,
            items: Vec::new(),
            coins: 0,
            display_case: None,
        }
    }
}

impl InventoryBuilder {
    pub fn max_capacity(mut self, max_capacity: usize) -> Self {
        self.max_capacity = max_capacity;
        self
    }

    pub fn items(mut self, items: Vec<Entity>) -> Self {
        self.items = items;
        self
    }

    pub fn coins(mut self, coins: u32) -> Self {
        self.coins = coins;
        self
    }

    pub fn display_case(mut self, display_case: Option<Entity>) -> Self {
        self.display_case = display_case;
        self
    }

    pub fn build(self) -> Inventory {
        let mut inventory = Inventory {
            max_capacity: self.max_capacity,
            items: VecDeque::new(),
            coins: self.coins,
            mainhand_index: None,
            offhand_index: None,
            display_case: self.display_case,
        };

        for item in self.items {
            inventory.add_item(item).unwrap(); // Assuming all items can fit for now.  Handle errors as needed
        }
        inventory
    }
}
