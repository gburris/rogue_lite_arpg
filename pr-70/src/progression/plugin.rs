use bevy::prelude::*;

use crate::progression::systems::handle_restart_trigger;

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_restart_trigger);
    }
}
