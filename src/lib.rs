// Module declarations - keep these at the top
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
