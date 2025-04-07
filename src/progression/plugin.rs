use bevy::prelude::*;

use crate::progression::systems::handle_restart_trigger;

use super::GameProgress;

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameProgress>()
            .add_observer(handle_restart_trigger);
    }
}
