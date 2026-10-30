pub mod constants;
mod damage_overlay;
mod game_over_screen;
mod load_screen;
pub mod plugin;
pub mod primitives;
mod start_screen;

pub mod prelude {
    pub use super::constants::{
        BACKGROUND_COLOR, DARK_GRAY_ALPHA_COLOR, DARK_GRAY_COLOR, FOOTER_HEIGHT,
    };
    pub use super::primitives::{menu_header, text, width};
}
