// Module declarations - keep these at the top
pub mod animation;
pub mod combat;
pub mod configuration;
pub mod debug;
pub mod despawn;
pub mod econ;
pub mod enemy;
pub mod items;
pub mod labels;
pub mod map;
pub mod movement;
pub mod npc;
pub mod player;
pub mod progression;
pub mod ui;

#[cfg(target_arch = "wasm32")]
use crate::configuration::plugins::WasmPlugins;

#[cfg(target_arch = "wasm32")]
use bevy::prelude::App;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    #[cfg(target_arch = "wasm32")]
    App::new().add_plugins(WasmPlugins).run();
}
