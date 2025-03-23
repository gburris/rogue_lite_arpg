// Module declarations - keep these at the top
pub mod ai;
pub mod animation;
pub mod character;
pub mod combat;
pub mod configuration;
pub mod controller;
pub mod despawn;
pub mod econ;
pub mod enemy;
pub mod items;
pub mod labels;
pub mod map;
pub mod npc;
pub mod player;
pub mod progression;
pub mod ui;

#[cfg(feature = "debug")]
pub mod debug;

#[cfg(target_arch = "wasm32")]
pub mod wasm {

    use crate::configuration::plugins::WasmPlugins;

    use bevy::prelude::App;

    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn start() {
        console_error_panic_hook::set_once();
        #[cfg(target_arch = "wasm32")]
        App::new().add_plugins(WasmPlugins).run();
    }
}
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
