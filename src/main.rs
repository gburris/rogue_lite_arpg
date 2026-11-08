// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;
#[cfg(feature = "dev_tools")]
use clap::Parser;

mod animation;
mod character;
mod combat;
mod configuration;
#[cfg(feature = "dev_tools")]
mod dev;
mod items;
mod lifespan;
mod menu;
mod ui;
mod world;

pub mod prelude {
    pub use super::animation::{AnimationData, AnimationIndices, AnimationTimer};
    pub use super::character::prelude::*;
    pub use super::combat::prelude::*;
    pub use super::configuration::prelude::*;
    #[cfg(feature = "dev_tools")]
    pub use super::dev::prelude::*;
    pub use super::items::prelude::*;
    pub use super::lifespan::{Lifespan, despawn_all, schedule_component_removal};
    pub use super::menu::prelude::*;
    pub use super::ui::prelude::*;
    pub use super::world::prelude::*;
}

fn main() {
    #[cfg(feature = "dev_tools")]
    let stress_args = dev::StressArgs::parse();

    let mut app = App::new();
    #[cfg(feature = "dev_tools")]
    app.insert_resource(stress_args);
    app.add_plugins(plugin);
    #[cfg(feature = "dev")]
    if dev::dump_schedules_if_requested(&mut app) {
        return;
    }
    app.run();
}

fn plugin(app: &mut App) {
    // Core systems
    app.add_plugins((
        animation::plugin,
        lifespan::plugin,
        configuration::plugin,
        combat::plugin,
    ));

    // Entity systems
    app.add_plugins((world::plugin, items::plugin, character::plugin));

    // UI
    app.add_plugins((ui::plugin, menu::plugin));

    #[cfg(feature = "dev_tools")]
    app.add_plugins(dev::plugin);
}
