use bevy::prelude::*;

use super::constants::GOLD_TEXT_COLOR;
use super::primitives::{gold_frame, splash_screen, text};
use crate::prelude::*;

#[derive(Component, Default, Clone)]
pub struct LoadScreen;

pub(super) fn scene() -> impl Scene {
    bsn! {
        @splash_screen(AppState::SpawnMap, "Loading Map", loading_badge(), "I'm loading")
        LoadScreen
        // Covers the map while it's built underneath.
        GlobalZIndex(1)
    }
}

fn loading_badge() -> impl Scene {
    bsn! {
        @gold_frame()
        Node {
            width: px(300),
            height: px(80),
            margin: px(20),
        }
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7))
        Children [
            @text("Loading Map", 48.0)
            TextColor(GOLD_TEXT_COLOR)
        ]
    }
}
