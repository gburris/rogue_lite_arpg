use bevy::prelude::*;

use crate::ui::{constants::TITLE_FONT_SIZE, primitives::gold_border};

#[derive(Component)]
pub struct LoadScreen;

#[derive(Component)]
pub struct AnimatedText;

pub fn spawn_load_screen(mut commands: Commands) {
    commands.spawn((
        LoadScreen,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            // Make sure there's no extra space on sides
            padding: UiRect::all(Val::Px(0.0)),
            margin: UiRect::all(Val::Px(0.0)),
            ..default()
        },
        // Darker background for more contrast
        BackgroundColor::from(Color::srgb(0.02, 0.01, 0.04)),
        GlobalZIndex(1),
        children![
            gold_border(),
            title_section(),
            body_section(),
            footer_section(),
            gold_border()
        ],
    ));
}

fn title_section() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(300.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            // Remove any potential spacing issues
            margin: UiRect::all(Val::Px(0.0)),
            padding: UiRect::all(Val::Px(0.0)),
            ..default()
        },
        children![(
            Node {
                width: Val::Auto,
                height: Val::Auto,
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::horizontal(Val::Px(40.0)),
                margin: UiRect::all(Val::Px(0.0)),
                ..default()
            },
            BorderColor(Color::srgb(0.8, 0.6, 0.2)),
            BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            children![(
                Text::new("Loading Instance"),
                TextFont {
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor::from(Color::srgb(0.9, 0.7, 0.2)),
                AnimatedText,
            )]
        )],
    )
}

fn body_section() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Px(0.0)),
            padding: UiRect::all(Val::Px(0.0)),
            ..default()
        },
        children![(
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
            children![(
                Text::new("Loading Instance"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor::from(Color::srgb(0.9, 0.8, 0.3)),
            )]
        )],
    )
}

fn footer_section() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(120.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            margin: UiRect::all(Val::Px(0.0)),
            ..default()
        },
        BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        children![(
            Text::new("I'm loading"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor::from(Color::srgb(0.7, 0.6, 0.5)),
        )],
    )
}

pub fn animate_text(time: Res<Time>, mut query: Query<&mut TextColor, With<AnimatedText>>) {
    for mut color in query.iter_mut() {
        let sine = (time.elapsed_secs() * 4.0).sin() * 0.4 + 0.6; // Increased frequency and amplitude
        *color = TextColor::from(Color::srgb(1.0 * sine, 0.5 * sine, 0.3 * sine));
    }
}
