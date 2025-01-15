use bevy::prelude::*;

use crate::{
    labels::states::{AppState, PausedState},
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
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(1),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            // Header Section
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(120.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor::from(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("GAME PAUSED"),
                        TextFont {
                            font_size: 80.0,
                            ..default()
                        },
                        Node::default(),
                    ));
                });

            // Body Section (transparent)
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(20.0),
                        padding: UiRect::vertical(Val::Px(40.0)),
                        ..default()
                    },
                    BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|body| {
                    // Equipment Button
                    spawn_menu_button(body, "EQUIPMENT", true);
                    // Additional menu buttons (placeholders)
                    spawn_menu_button(body, "INVENTORY", true);
                    spawn_menu_button(body, "SKILLS", true);
                });

            // Footer Section
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(80.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(40.0)),
                        ..default()
                    },
                    BackgroundColor::from(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|footer| {
                    // Player Stats
                    footer
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(20.0),
                            ..default()
                        },))
                        .with_children(|stats| {
                            stats.spawn((
                                Text::new("Level: 1"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                Node::default(),
                            ));
                            stats.spawn((
                                Text::new("Health: 100.0 / 100.0"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                Node::default(),
                            ));
                        });

                    // Exit Instructions
                    footer.spawn((
                        Text::new("Press ESC to unpause"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        Node::default(),
                    ));
                });
        });
}

fn spawn_menu_button(parent: &mut ChildBuilder, text: &str, is_equipment: bool) {
    let mut button = parent.spawn((
        Button,
        Node {
            width: Val::Px(300.0),
            height: Val::Px(60.0),
            border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::srgb(0.8, 0.8, 0.8)),
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
    ));

    if is_equipment {
        button.insert(EquipmentMenuButton);
    }

    button.with_children(|button| {
        button.spawn((
            Text::new(text),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            Node::default(),
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
        BackgroundColor::from(Color::BLACK.with_alpha(0.9)),
        Visibility::Visible,
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
    mut pause_state: ResMut<NextState<PausedState>>,
) {
    for (interaction, _) in &mut equipment_button_query {
        //warn!("handle_equipment_button_pressed interaction");
        if *interaction == Interaction::Pressed {
            warn!("handle_equipment_button_pressed");
            pause_state.set(PausedState::Equipment);
        }
    }
}

pub fn handle_ui_inputs(mut commands: Commands, mut keyboard_input: ResMut<ButtonInput<KeyCode>>) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        warn!("ui_inputs, enter");
        commands.trigger(PauseInputEvent);
    }
}

pub fn on_pause_input(
    _: Trigger<PauseInputEvent>, // Access keyboard input
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match state.get() {
        AppState::Paused => {
            warn!("Currently paused, transitioning to playing");
            next_state.set(AppState::Playing)
        }
        _ => {
            warn!("Not currently paused, transitioning to paused");
            next_state.set(AppState::Paused);
        }
    }
}
