use bevy::prelude::*;

use crate::{
    prelude::AppState,
    ui_primitives::{
        constants::TITLE_FONT_SIZE,
        primitives::{gold_border, text},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::StartScreen), spawn_splash_screen)
        .add_systems(
            Update,
            (handle_button_interactions, animate_text).run_if(in_state(AppState::StartScreen)),
        );
}

#[derive(Component)]
struct SplashScreen;

#[derive(Component)]
struct StartScreenButton;

#[derive(Component)]
struct AnimatedText;

pub fn spawn_splash_screen(mut commands: Commands) {
    commands.spawn((
        SplashScreen,
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

// Enhanced button system with more dramatic hover effects
fn handle_button_interactions(
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

fn animate_text(time: Res<Time>, mut query: Query<&mut TextColor, With<AnimatedText>>) {
    for mut color in query.iter_mut() {
        let sine = (time.elapsed_secs() * 4.0).sin() * 0.4 + 0.6; // Increased frequency and amplitude
        *color = TextColor::from(Color::srgb(1.0 * sine, 0.5 * sine, 0.3 * sine));
    }
}
