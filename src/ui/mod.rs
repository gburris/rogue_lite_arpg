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
        BACKGROUND_COLOR, BODY_FONT_SIZE, CAPTION_FONT_SIZE, DARK_GRAY_ALPHA_COLOR,
        DARK_GRAY_COLOR, FOOTER_HEIGHT, MUTED_TEXT_COLOR, ROW_COLOR, SMALL_FONT_SIZE,
    };
    pub use super::primitives::{
        ButtonBorderColors, ButtonColors, MENU_BUTTON_COLORS, button, menu_button, menu_footer,
        menu_header, menu_screen, muted_text, panel, spacer, text, unpause_hint, width,
    };
}

pub(super) fn plugin(app: &mut App) {
    // Loading, start and game over screens
    app.add_systems(OnEnter(AppState::SpawnMap), load_screen::scene.spawn())
        .add_systems(OnEnter(AppState::StartScreen), start_screen::scene.spawn())
        .add_systems(OnEnter(AppState::GameOver), game_over_screen::scene.spawn());

    // Shared widget behavior
    app.add_systems(
        Update,
        (primitives::style_buttons, primitives::pulse_text).in_set(MainSystems::Shared),
    );

    // Heal and damage overlays
    app.add_observer(damage_overlay::on_damage_overlay_amount)
        .add_observer(damage_overlay::on_healing_overlay_amount);
}
