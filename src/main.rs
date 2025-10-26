// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;

mod animation;
pub mod character;
pub mod combat;
mod configuration;
mod items;
pub mod progression;
pub mod ui;
pub mod utility;
mod world;

pub mod prelude {
    pub use super::animation::*;
    pub use super::character::prelude::*;
    pub use super::configuration::prelude::*;
    pub use super::items::prelude::*;
    pub use super::utility::*;
    pub use super::world::prelude::*;
}

fn main() {
    App::new().add_plugins(plugin).run();
}

fn plugin(app: &mut App) {
    // Core systems
    app.add_plugins((
        animation::plugin,
        utility::plugin,
        configuration::plugin,
        combat::plugin,
        progression::plugin::ProgressionPlugin,
    ));

    // Entity systems
    app.add_plugins((world::plugin, items::plugin, character::CharacterPlugin));

    // UI
    app.add_plugins(ui::plugin::UIPlugin);
}
