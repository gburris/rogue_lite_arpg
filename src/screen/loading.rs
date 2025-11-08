use bevy::prelude::*;

use crate::{
    prelude::AppState,
    ui_primitives::{
        constants::TITLE_FONT_SIZE,
        primitives::{gold_border, text},
    },
};

pub(super) fn plugin(app: &mut App) {
    //Loading screen
    app.add_systems(OnEnter(AppState::SpawnZone), spawn_loading_screen)
        .add_systems(Update, animate_text.run_if(in_state(AppState::SpawnZone)));
}

#[derive(Component)]
struct LoadingScreen;

#[derive(Component)]
struct AnimatedText;

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        LoadingScreen,
        DespawnOnExit(AppState::SpawnZone),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            // Make sure there's no extra space on sides
            padding: px(0.0).all(),
            margin: px(0.0).all(),
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
            width: percent(100.0),
            height: px(300.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            // Remove any potential spacing issues
            margin: px(0.0).all(),
            padding: px(0.0).all(),
            ..default()
        },
        children![(
            Node {
                width: auto(),
                height: auto(),
                border: px(2.0).all(),
                padding: px(40.0).horizontal(),
                margin: px(0.0).all(),
                ..default()
            },
            BorderColor::all(Color::srgb(0.8, 0.6, 0.2)),
            BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            children![(
                text("Loading Instance", TITLE_FONT_SIZE),
                TextColor::from(Color::srgb(0.9, 0.7, 0.2)),
                AnimatedText,
            )]
        )],
    )
}

fn body_section() -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: px(0.0).all(),
            padding: px(0.0).all(),
            ..default()
        },
        children![(
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
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
            children![(
                text("Loading Instance", 48.0),
                TextColor::from(Color::srgb(0.9, 0.8, 0.3)),
            )]
        )],
    )
}

fn footer_section() -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: px(120.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: px(20.0).all(),
            margin: px(0.0).all(),
            ..default()
        },
        BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        children![(
            text("I'm loading", 24.0),
            TextColor::from(Color::srgb(0.7, 0.6, 0.5)),
        )],
    )
}

fn animate_text(time: Res<Time>, mut query: Query<&mut TextColor, With<AnimatedText>>) {
    for mut color in query.iter_mut() {
        let sine = (time.elapsed_secs() * 4.0).sin() * 0.4 + 0.6; // Increased frequency and amplitude
        *color = TextColor::from(Color::srgb(1.0 * sine, 0.5 * sine, 0.3 * sine));
    }
}
