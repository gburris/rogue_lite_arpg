use bevy::prelude::*;

use crate::{items::ItemName, player::Inventory};

use super::PrintInventoryEvent;

pub fn print_inventory(
    _: Trigger<PrintInventoryEvent>,
    query_inventory: Query<&Inventory>,
    item_query: Query<&ItemName>,
) {
    if let Ok(player_inventory) = query_inventory.get_single() {
        println!("Inventory contains:");

        for (slot, item_entity) in &player_inventory.items {
            println!("Item in inventory: in slot {}", slot);
            let item_name = item_query.get(*item_entity);
            if let Ok(item_name) = item_name {
                println!("Item name: {:?}", item_name);
            } else {
                println!("Failed to get item name for slot {}", slot);
            }
        }
    }
}
