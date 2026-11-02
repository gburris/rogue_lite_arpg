// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;

mod animation;
mod character;
mod combat;
mod configuration;
#[cfg(any(feature = "dev", feature = "stress", feature = "profile"))]
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
    #[cfg(feature = "stress")]
    pub use super::dev::prelude::*;
    pub use super::items::prelude::*;
    pub use super::lifespan::{Lifespan, despawn_all, schedule_component_removal};
    pub use super::menu::prelude::*;
    pub use super::ui::prelude::*;
    pub use super::world::prelude::*;
}

fn main() {
    let mut app = App::new();
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

    #[cfg(any(feature = "dev", feature = "stress", feature = "profile"))]
    app.add_plugins(dev::plugin);
}
