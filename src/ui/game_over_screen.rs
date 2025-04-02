use bevy::prelude::*;

use crate::{
    configuration::time_control::RestartEvent, labels::states::AppState, map::CleanupZone,
    player::Player,
};

use super::{constants::TITLE_FONT_SIZE, primitives::text};

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct RestartButton;

pub fn create(mut commands: Commands) {
    commands.spawn((
        GameOverScreen,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::top(Val::Px(200.0)),
            row_gap: Val::Px(20.),
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
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(5.0)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(Color::BLACK),
                BorderRadius::MAX,
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                children![Text::new("Restart")]
            )
        ],
    ));
}

pub fn despawn_game_over_screen(
    mut commands: Commands,
    game_over_screen: Single<Entity, With<GameOverScreen>>,
) {
    // Despawn game over screen
    commands.entity(*game_over_screen).despawn();
}

//Query the player level, add it to the restart event
pub fn handle_restart_button(
    mut restart_query: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut game_state: ResMut<NextState<AppState>>,
    player: Single<&Player>,
    mut commands: Commands,
) {
    for interaction in &mut restart_query {
        if *interaction == Interaction::Pressed {
            commands.trigger(RestartEvent {
                player_level: player.get_level(),
            });
            game_state.set(AppState::SpawnPlayer);
        }
    }
}

pub fn on_restart_event_cleanup_zone(_: Trigger<RestartEvent>, mut commands: Commands) {
    commands.trigger(CleanupZone);
}
