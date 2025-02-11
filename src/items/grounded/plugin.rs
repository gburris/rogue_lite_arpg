use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{
    handle_item_to_ground::handle_item_ground_transition,
    on_grounded_item_interaction::on_grounded_item_input_interaction,
    update_autoloot::update_autoloot_currency, update_grounded_items::update_grounded_items,
    update_magnets::update_grounded_magnets,
};

pub struct GroundedPlugin;

impl Plugin for GroundedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_grounded_items,
                update_autoloot_currency,
                update_grounded_magnets,
            )
                .in_set(InGameSet::Simulation),
        )
        .add_observer(handle_item_ground_transition)
        .add_observer(on_grounded_item_input_interaction);
    }
}
