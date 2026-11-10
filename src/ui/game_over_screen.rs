use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::constants::TITLE_FONT_SIZE;
use super::primitives::{gold_button, screen, text};
use crate::prelude::*;

#[derive(Component, Default, Clone)]
pub struct GameOverScreen;

pub(super) fn scene() -> impl Scene {
    bsn! {
        @screen(AppState::GameOver)
        GameOverScreen
        Node {
            align_items: AlignItems::Center,
            padding: {px(200).top()},
            row_gap: px(20),
        }
        BackgroundColor(Color::BLACK)
        // Render above in-game UI such as player health and score.
        GlobalZIndex(1)
        Children [
            @text("Game Over!", TITLE_FONT_SIZE)
            --
            @gold_button("Restart", 32.0)
            Node {
                width: px(200),
                height: px(65),
                border_radius: BorderRadius::MAX,
            }
            on(on_restart)
        ]
    }
}

/// Passes the player's current level to the next run, despawns everything and starts again.
fn on_restart(_: On<Activate>, mut commands: Commands, player: Single<&Player>) {
    commands.trigger(Restart {
        player_level: player.level(),
    });
}
