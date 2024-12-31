use crate::systems::{cleanup_health_displays, spawn_health_displays, update_health_displays};
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        log::info!("UI Plugin Plugin! added!");
        app.add_systems(
            Update,
            (
                spawn_health_displays,
                update_health_displays,
                cleanup_health_displays,
            ),
        );
    }
}
