use avian2d::prelude::*;
use bevy::prelude::*;

use crate::labels::states::GameState;

#[derive(Component)]
pub struct PauseMenu;

pub fn create(mut commands: Commands) {
    commands
        .spawn((
            PauseMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor::from(Color::BLACK.with_alpha(0.9)), // want to allow game to be seen in background
            Visibility::Hidden, // created pause menu should be hidden to start
            // render this above in-game UI such as player health and score
            GlobalZIndex(1),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Paused"),
                TextFont {
                    font_size: 70.0,
                    ..default()
                },
                Node {
                    top: Val::Percent(-20.0),
                    ..default()
                },
            ));
        });
}

// Make pause menu visible when we enter the state
pub fn on_pause(
    mut pause_menu_query: Query<&mut Visibility, With<PauseMenu>>,
    mut time: ResMut<Time<Physics>>,
) {
    if let Ok(mut pause_menu_visibility) = pause_menu_query.get_single_mut() {
        *pause_menu_visibility = Visibility::Visible;
    }

    time.pause();
}

// Cleanup pause menu once we return to game, set it to hidden
pub fn on_resume_game(
    mut pause_menu_query: Query<&mut Visibility, With<PauseMenu>>,
    mut time: ResMut<Time<Physics>>,
) {
    if let Ok(mut pause_menu_visibility) = pause_menu_query.get_single_mut() {
        *pause_menu_visibility = Visibility::Hidden;
    }

    time.unpause();
}

// Watcher system to determine when to go back to the game
pub fn return_to_game(
    mut game_state: ResMut<NextState<GameState>>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        game_state.set(GameState::Playing);
    }
}
