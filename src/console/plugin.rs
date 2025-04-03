use bevy::{
    ecs::system::SystemState,
    prelude::*,
    scene::ron::{self, ser::PrettyConfig},
    tasks::{block_on, poll_once, IoTaskPool},
};

use super::{setup_console, update_console};

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_console)
            .add_systems(FixedUpdate, update_console);
    }
}
