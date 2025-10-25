// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;

pub mod animation;
pub mod character;
pub mod combat;
pub mod configuration;
pub mod items;
pub mod progression;
pub mod ui;
pub mod utility;
mod world;

pub mod prelude {
    pub use super::character::prelude::*;
    pub use super::configuration::prelude::*;
    pub use super::world::prelude::*;
}

fn main() {
    App::new().add_plugins(plugin).run();
}

fn plugin(app: &mut App) {
    // Core systems
    app.add_plugins((
        utility::plugin,
        configuration::plugin,
        combat::CombatPlugin,
        progression::plugin::ProgressionPlugin,
    ));

    // Entity systems
    app.add_plugins((world::plugin, items::plugin, character::CharacterPlugin));

    // UI
    app.add_plugins(ui::plugin::UIPlugin);
}
