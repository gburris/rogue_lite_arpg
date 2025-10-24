// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;

use crate::configuration::GamePlugin;

pub mod animation;
pub mod character;
pub mod combat;
pub mod configuration;
pub mod items;
pub mod map;
pub mod progression;
pub mod ui;
pub mod utility;
pub mod world;

pub mod prelude {
    pub use super::character::prelude::*;
    pub use super::configuration::prelude::*;
    pub use super::world::prelude::*;
}

fn main() {
    App::new().add_plugins(GamePlugin).run();
}
