use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::on_grounded_item_interaction::on_grounded_item_input_interaction;

pub struct GroundedPlugin;

impl Plugin for GroundedPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_grounded_item_input_interaction);
    }
}
