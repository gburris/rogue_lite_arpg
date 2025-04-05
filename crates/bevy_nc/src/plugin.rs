mod handlers;
use bevy_app::App;

use crate::bevy::prelude::{FixedUpdate, Plugin, Startup};

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, handlers::setup_console)
            .add_systems(FixedUpdate, handlers::update_console);
    }
}
