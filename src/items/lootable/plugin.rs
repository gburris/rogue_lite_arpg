use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{
    handle_item_to_ground::handle_item_ground_transition,
    on_lootable_item_interaction::on_lootable_item_input_interaction,
    update_autoloot::update_autoloot_currency, update_lootable_items::update_lootable_items,
    update_magnets::update_magnets,
};

pub struct LootablePlugin;

impl Plugin for LootablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_lootable_items,
                update_autoloot_currency,
                update_magnets,
            )
                .in_set(InGameSet::Simulation),
        )
        .add_observer(handle_item_ground_transition)
        .add_observer(on_lootable_item_input_interaction);
    }
}
