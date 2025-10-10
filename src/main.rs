use bevy::prelude::*;

use crate::configuration::plugins::GamePlugins;

pub mod animation;
pub mod character;
pub mod combat;
pub mod configuration;
pub mod economy;
pub mod items;
pub mod labels;
pub mod map;
pub mod progression;
pub mod ui;
pub mod utility;

pub mod prelude {
    pub use crate::character::prelude::*;
}

fn main() {
    App::new().add_plugins(GamePlugins).run();
}
