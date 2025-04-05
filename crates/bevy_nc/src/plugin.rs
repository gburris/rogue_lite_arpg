//! [`handlers`]
mod handlers;
mod tcp;
use handlers::HostConsoleAddr;

use crate::bevy::prelude::*;

pub struct NetConsolePlugin;
impl Plugin for NetConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HostConsoleAddr>()
            .add_systems(Startup, handlers::setup_host_console)
            .add_systems(FixedUpdate, handlers::update_host_console);
    }
}
