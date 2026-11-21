mod primitives;
mod screens;

use bevy::prelude::*;

use crate::prelude::*;

pub mod prelude {
    pub use super::primitives::{
        BACKGROUND_COLOR, BODY_FONT_SIZE, ButtonBorderColors, ButtonColors, CAPTION_FONT_SIZE,
        DARK_GRAY_ALPHA_COLOR, DARK_GRAY_COLOR, FOOTER_HEIGHT, MENU_BUTTON_COLORS,
        MUTED_TEXT_COLOR, ROW_COLOR, SMALL_FONT_SIZE, button, menu_button, menu_footer,
        menu_header, menu_screen, muted_text, panel, spacer, text, unpause_hint, width,
    };
}

pub(super) fn plugin(app: &mut App) {
    // Loading, start and game over screens
    app.add_systems(OnEnter(AppState::SpawnMap), screens::load_screen.spawn())
        .add_systems(
            OnEnter(AppState::StartScreen),
            screens::start_screen.spawn(),
        )
        .add_systems(
            OnEnter(AppState::GameOver),
            screens::game_over_screen.spawn(),
        );

    // Shared widget behavior
    app.add_systems(
        Update,
        (primitives::style_buttons, primitives::pulse_text).in_set(MainSystems::Shared),
    );
}
