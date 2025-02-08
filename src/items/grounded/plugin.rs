use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::handle_collisions::handle_grounded_item_collision;

pub struct GroundedPlugin;

impl Plugin for GroundedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((handle_grounded_item_collision,)).in_set(InGameSet::Simulation),
        );
    }
}
