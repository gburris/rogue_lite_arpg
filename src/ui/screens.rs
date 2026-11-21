use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::primitives::{
    GOLD_TEXT_COLOR, TITLE_FONT_SIZE, gold_button, gold_frame, screen, splash_screen, text,
};
use crate::prelude::*;

#[derive(Component, Default, Clone)]
pub struct LoadScreen;

pub(super) fn load_screen() -> impl Scene {
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

#[derive(Component, Default, Clone)]
pub struct StartScreen;

pub(super) fn start_screen() -> impl Scene {
    bsn! {
        @splash_screen(
            AppState::StartScreen,
            "Baba Yaga",
            begin_button(),
            "She is a mysterious witch and ogress from Slavic folklore",
        )
        StartScreen
    }
}

fn begin_button() -> impl Scene {
    bsn! {
        @gold_button("BEGIN", 48.0)
        Node {
            width: px(300),
            height: px(80),
            margin: px(20),
        }
        on(|_: On<Activate>, mut next_app_state: ResMut<NextState<AppState>>| {
            next_app_state.set(AppState::AssetLoading);
        })
    }
}

#[derive(Component, Default, Clone)]
pub struct GameOverScreen;

pub(super) fn game_over_screen() -> impl Scene {
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
