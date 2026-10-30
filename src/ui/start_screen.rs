use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button, observe};

use super::constants::TITLE_FONT_SIZE;
use super::primitives::{gold_border, text};
use crate::prelude::*;

#[derive(Component)]
pub struct StartScreen;

#[derive(Component)]
pub struct StartScreenButton;

#[derive(Component)]
pub struct AnimatedText;

pub(super) fn spawn(mut commands: Commands) {
    commands.spawn((
        StartScreen,
        DespawnOnExit(AppState::StartScreen),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        // Darker background for more contrast
        BackgroundColor::from(Color::srgb(0.02, 0.01, 0.04)),
        children![
            gold_border(),
            start_screen_title(),
            start_screen_body(),
            start_screen_footer(),
            gold_border(),
        ],
    ));
}

fn start_screen_title() -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: px(300.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Node {
                width: auto(),
                height: auto(),
                border: px(2.0).all(),
                padding: px(40.0).all(),
                ..default()
            },
            BorderColor::all(Color::srgb(0.8, 0.6, 0.2)),
            BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            children![(
                text("Baba Yaga", TITLE_FONT_SIZE),
                TextColor::from(Color::srgb(0.9, 0.7, 0.2)),
                AnimatedText,
            )]
        ),],
    )
}

fn start_screen_body() -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            StartScreenButton,
            Button,
            Hovered::default(),
            observe(on_start_button_activated),
            Node {
                width: px(300.0),
                height: px(80.0),
                border: px(2.0).all(),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: px(20.0).all(),
                ..default()
            },
            BorderColor::all(Color::srgb(0.8, 0.6, 0.2)),
            BackgroundColor::from(Color::srgba(0.1, 0.1, 0.1, 0.7)),
            children![(
                text("BEGIN", 48.0),
                TextColor::from(Color::srgb(0.9, 0.8, 0.3)),
            )]
        )],
    )
}

fn start_screen_footer() -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: px(120.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: px(20.0).all(),
            ..default()
        },
        BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        children![(
            text(
                "She is a mysterious witch and ogress from Slavic folklore",
                24.0
            ),
            TextColor::from(Color::srgb(0.7, 0.6, 0.5)),
        )],
    )
}

fn on_start_button_activated(_: On<Activate>, mut next_app_state: ResMut<NextState<AppState>>) {
    next_app_state.set(AppState::AssetLoading);
}

// Hover and press styling for the start button.
pub(super) fn style_start_button(
    mut button_query: Query<
        (
            Has<Pressed>,
            &Hovered,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<StartScreenButton>,
    >,
) {
    for (pressed, hovered, mut background_color, mut border_color) in &mut button_query {
        if pressed {
            *background_color = BackgroundColor::from(Color::srgba(0.3, 0.2, 0.1, 0.9));
            *border_color = BorderColor::from(Color::srgb(1.0, 0.8, 0.3));
        } else if hovered.get() {
            *background_color = BackgroundColor::from(Color::srgba(0.2, 0.15, 0.1, 0.8));
            *border_color = BorderColor::from(Color::srgb(1.0, 0.8, 0.3));
        } else {
            *background_color = BackgroundColor::from(Color::srgba(0.1, 0.1, 0.1, 0.7));
            *border_color = BorderColor::from(Color::srgb(0.8, 0.6, 0.2));
        }
    }
}

pub(super) fn animate_text(time: Res<Time>, mut query: Query<&mut TextColor, With<AnimatedText>>) {
    for mut color in &mut query {
        // Increased frequency and amplitude.
        let sine = (time.elapsed_secs() * 4.0).sin() * 0.4 + 0.6;
        *color = TextColor::from(Color::srgb(1.0 * sine, 0.5 * sine, 0.3 * sine));
    }
}
