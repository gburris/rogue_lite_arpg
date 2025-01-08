use bevy::prelude::*;

use crate::player::Inventory;

// System to print the inventory contents when the 'I' key is pressed
pub fn print_inventory(query_inventory: Query<&Inventory>) {
    for inventory in query_inventory.iter() {
        println!("Inventory contains:");

        for (item_name, item) in &inventory.items {
            println!("Item in inventory: {}", item_name);
            // Print all stats of the item
            for (stat_name, stat_value) in &item.stats {
                println!("  - {}: {}", stat_name, stat_value);
            }
        }
    }
}
