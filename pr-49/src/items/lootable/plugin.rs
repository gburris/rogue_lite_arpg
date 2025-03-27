use bevy::prelude::*;

use crate::{items::on_item_added, labels::sets::InGameSet};

use super::{
    handle_item_to_ground::handle_item_ground_transition,
    update_lootable_items::glow_and_rotate_lootables, update_magnets::update_magnet_locations,
};

pub struct LootablePlugin;

impl Plugin for LootablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (glow_and_rotate_lootables.in_set(InGameSet::Vfx),))
            .add_systems(
                FixedUpdate,
                update_magnet_locations.in_set(InGameSet::Physics),
            )
            .add_observer(on_item_added)
            .add_observer(handle_item_ground_transition);
    }
}
