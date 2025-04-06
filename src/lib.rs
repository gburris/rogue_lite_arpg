// Module declarations - keep these at the top
pub mod ai;
pub mod animation;
pub mod character;
pub mod combat;
pub mod configuration;
pub mod despawn;
pub mod economy;
pub mod enemy;
pub mod items;
pub mod labels;
pub mod map;
pub mod npc;
pub mod player;
pub mod progression;
pub mod ui;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    pub use crate::configuration::plugins::WasmPlugins;
    pub use bevy::prelude::App;
    pub use wasm_bindgen::prelude::*;
    console_error_panic_hook::set_once();
    App::new().add_plugins(WasmPlugins).run();
}
