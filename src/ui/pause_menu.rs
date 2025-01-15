use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    labels::states::{GameState, PausedState},
    player::systems::PauseInputEvent,
};

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct PauseScreen;

#[derive(Component)]
pub struct EquipmentMenuButton;

pub fn spawn_main_menu(mut commands: Commands) {
    warn!("spawn_main_menu called");
    commands
        .spawn((
            MainMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor::from(Color::BLACK.with_alpha(0.9)), // want to allow game to be seen in background
            Visibility::Visible, // created pause menu should be hidden to start
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
            parent.spawn((
                Text::new("Equipment"),
                EquipmentMenuButton,
                Button,
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                Node {
                    right: Val::Percent(20.0),
                    ..default()
                },
            ));
        });
}

pub fn spawn_pause_screen(mut commands: Commands) {
    warn!("spawn_pause_screen called");
    commands.spawn((
        PauseScreen,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor::from(Color::BLACK.with_alpha(0.9)), // want to allow game to be seen in background
        Visibility::Visible, // created pause menu should be hidden to start
        // render this above in-game UI such as player health and score
        GlobalZIndex(1),
    ));
}

pub fn despawn_pause_screen(
    mut commands: Commands,
    pause_menu_background_query: Query<Entity, With<PauseScreen>>,
) {
    warn!("despawn_pause_screen called");
    for entity in pause_menu_background_query.iter() {
        commands.entity(entity).despawn();
    }
}

// Despawn entities with the MainMenu component
pub fn despawn_main_menu(
    mut commands: Commands,
    pause_menu_background_query: Query<Entity, With<MainMenu>>,
) {
    warn!("despawn_main_menu called");
    for entity in pause_menu_background_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_equipment_button_pressed(
    mut equipment_button_query: Query<(&Interaction, &mut EquipmentMenuButton)>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, _) in &mut equipment_button_query {
        //warn!("handle_equipment_button_pressed interaction");
        if *interaction == Interaction::Pressed {
            warn!("handle_equipment_button_pressed");
            game_state.set(GameState::Paused(PausedState::Equipment));
        }
    }
}

// Watcher system to determine when to go back to the game
pub fn return_to_game(
    mut game_state: ResMut<NextState<GameState>>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        warn!("return_to_game triggered");
        game_state.set(GameState::Playing);
    }
}

pub fn set_default_menu_state(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Paused(PausedState::MainMenu));
}

pub fn ui_inputs(mut commands: Commands, mut keyboard_input: ResMut<ButtonInput<KeyCode>>) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        warn!("ui_inputs, enter");
        commands.trigger(PauseInputEvent);
        return;
    }
}

pub fn on_pause_input(
    _: Trigger<PauseInputEvent>, // Access keyboard input
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Check if we're currently in any paused state
    match state.get() {
        // If we're in any paused state, transition to exit
        GameState::Paused(_) => {
            warn!("Currently paused, transitioning to exit state");
            next_state.set(GameState::Paused(PausedState::Exit));
        }
        // If we're not paused, begin pause sequence
        _ => {
            warn!("Not currently paused, transitioning to enter state");
            next_state.set(GameState::Paused(PausedState::Enter));
        }
    }
}
