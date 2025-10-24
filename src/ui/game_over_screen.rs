use bevy::prelude::*;

use crate::{
    map::CleanupZone,
    prelude::AppState,
    prelude::{Player, RestartEvent},
};

use super::{constants::TITLE_FONT_SIZE, primitives::text};

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct RestartButton;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        GameOverScreen,
        DespawnOnExit(AppState::GameOver),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: px(200.0).top(),
            row_gap: px(20.),
            ..default()
        },
        BackgroundColor::from(Color::BLACK.with_alpha(1.0)), // want to allow game to be seen in background
        // render this above in-game UI such as player health and score
        GlobalZIndex(1),
        children![
            text("Game Over!", TITLE_FONT_SIZE),
            (
                Button,
                RestartButton,
                Node {
                    width: px(150.0),
                    height: px(65.0),
                    border: px(5.0).all(),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::BLACK),
                BorderRadius::MAX,
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                children![Text::new("Restart"), Observer::new(on_restart_clicked)]
            )
        ],
    ));
}

/// Passes players current level to the next instance of the game, despawns everything and starts again
fn on_restart_clicked(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<AppState>>,
    player: Single<&Player>,
) {
    commands.trigger(RestartEvent {
        player_level: player.get_level(),
    });
    game_state.set(AppState::SpawnPlayer);
}

pub fn on_restart_event_cleanup_zone(_: On<RestartEvent>, mut commands: Commands) {
    commands.trigger(CleanupZone);
}
