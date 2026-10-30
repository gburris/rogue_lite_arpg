mod constants;
mod damage_overlay;
mod game_over_screen;
mod load_screen;
mod primitives;
mod start_screen;

use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::constants::{
        BACKGROUND_COLOR, DARK_GRAY_ALPHA_COLOR, DARK_GRAY_COLOR, FOOTER_HEIGHT,
    };
    pub use super::primitives::{menu_header, text, width};
}

pub(super) fn plugin(app: &mut App) {
    // Loading screen
    app.add_systems(OnEnter(AppState::SpawnMap), load_screen::spawn)
        .add_systems(
            Update,
            (load_screen::animate_text).run_if(in_state(AppState::SpawnMap)),
        );

    // Start screen
    app.add_systems(OnEnter(AppState::StartScreen), start_screen::spawn)
        .add_systems(
            Update,
            (start_screen::style_start_button, start_screen::animate_text)
                .run_if(in_state(AppState::StartScreen)),
        );

    // Heal and damage overlays
    app.add_observer(damage_overlay::on_damage_overlay_amount)
        .add_observer(damage_overlay::on_healing_overlay_amount);

    // Game over systems
    app.add_systems(OnEnter(AppState::GameOver), game_over_screen::spawn);
}
