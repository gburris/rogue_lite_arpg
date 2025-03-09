use crate::labels::states::AppState;
use bevy::prelude::*;

use super::constants::TITLE_FONT_SIZE;

#[derive(Component)]
pub struct StartScreen;

#[derive(Component)]
pub struct StartScreenButton;

#[derive(Component)]
pub struct AnimatedText;

pub fn spawn_start_screen(mut commands: Commands) {
    commands
        .spawn((
            StartScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            // Darker background for more contrast
            BackgroundColor::from(Color::srgb(0.02, 0.01, 0.04)),
        ))
        .with_children(|parent| {
            // Top gold border
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor::from(Color::srgb(0.8, 0.6, 0.2)),
            ));

            // Title Section
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(300.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|header| {
                    // Title container with border
                    header
                        .spawn((
                            Node {
                                width: Val::Auto,
                                height: Val::Auto,
                                border: UiRect::all(Val::Px(2.0)),
                                padding: UiRect::horizontal(Val::Px(40.0)),
                                ..default()
                            },
                            BorderColor(Color::srgb(0.8, 0.6, 0.2)),
                            BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                        ))
                        .with_children(|title_container| {
                            title_container.spawn((
                                Text::new("Baba Yaga"),
                                TextFont {
                                    font_size: TITLE_FONT_SIZE,
                                    ..default()
                                },
                                TextColor::from(Color::srgb(0.9, 0.7, 0.2)),
                                AnimatedText,
                            ));
                        });
                });

            // Center content section
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|body| {
                    // Begin button
                    body.spawn((
                        StartScreenButton,
                        Button,
                        Node {
                            width: Val::Px(300.0),
                            height: Val::Px(80.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(0.8, 0.6, 0.2)),
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("BEGIN"),
                            TextFont {
                                font_size: 48.0,
                                ..default()
                            },
                            TextColor::from(Color::srgb(0.9, 0.8, 0.3)),
                        ));
                    });
                });

            // Footer
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(120.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                ))
                .with_children(|footer| {
                    footer.spawn((
                        Text::new("She is a mysterious witch and ogress from Slavic folklore"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor::from(Color::srgb(0.7, 0.6, 0.5)),
                    ));
                });

            // Bottom gold border
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor::from(Color::srgb(0.8, 0.6, 0.2)),
            ));
        });
}

pub fn despawn_start_screen(
    mut commands: Commands,
    start_screen_query: Query<Entity, With<StartScreen>>,
) {
    debug!("despawn_start_screen called");
    for entity in start_screen_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// Enhanced button system with more dramatic hover effects
pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<StartScreenButton>),
    >,
    mut game_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = Color::srgba(0.3, 0.2, 0.1, 0.9).into();
                *border_color = Color::srgb(1.0, 0.8, 0.3).into();
                game_state.set(AppState::AssetLoading);
            }
            Interaction::Hovered => {
                *bg_color = Color::srgba(0.2, 0.15, 0.1, 0.8).into();
                *border_color = Color::srgb(1.0, 0.8, 0.3).into();
            }
            Interaction::None => {
                *bg_color = Color::srgba(0.1, 0.1, 0.1, 0.7).into();
                *border_color = Color::srgb(0.8, 0.6, 0.2).into();
            }
        }
    }
}

pub fn animate_text(time: Res<Time>, mut query: Query<&mut TextColor, With<AnimatedText>>) {
    for mut color in query.iter_mut() {
        let sine = (time.elapsed_secs() * 4.0).sin() * 0.4 + 0.6; // Increased frequency and amplitude
        *color = TextColor::from(Color::srgb(1.0 * sine, 0.5 * sine, 0.3 * sine));
    }
}
