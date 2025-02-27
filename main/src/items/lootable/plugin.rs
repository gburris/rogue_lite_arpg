use bevy::prelude::*;

use crate::{items::on_item_added, labels::sets::InGameSet};

use super::{
    handle_item_to_ground::handle_item_ground_transition,
    update_lootable_items::update_lootable_items, update_magnets::update_magnets,
};

pub struct LootablePlugin;

impl Plugin for LootablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_lootable_items, update_magnets).in_set(InGameSet::Simulation),
        )
        .add_observer(on_item_added)
        .add_observer(handle_item_ground_transition);
    }
}
