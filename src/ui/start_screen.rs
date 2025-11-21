use bevy::prelude::*;

use crate::{
    prelude::AppState,
    ui::{
        constants::{color, font_size},
        element::{Element, node},
    },
};

use super::primitives::text;

#[derive(Component)]
pub struct StartScreen;

#[derive(Component)]
pub struct StartScreenButton;

#[derive(Component)]
pub struct AnimatedText;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        StartScreen,
        DespawnOnExit(AppState::StartScreen),
        Element::builder(
            node()
                .width(percent(100.0))
                .height(percent(100.0))
                .flex_direction(FlexDirection::Column)
                .build(),
        )
        .background_color(color::LOAD_SCREEN_BACKGROUND)
        .build(),
        children![
            start_screen_title(),
            start_screen_body(),
            start_screen_footer(),
        ],
    ));
}

fn start_screen_title() -> impl Bundle {
    (
        Element::builder(
            node()
                .width(percent(100.0))
                .height(px(300.0))
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .build(),
        )
        .build(),
        children![(
            Element::builder(
                node()
                    .width(auto())
                    .height(auto())
                    .border(px(2.0).all())
                    .padding(px(40.0).all())
                    .build(),
            )
            .background_color(color::BLACK.with_alpha(0.3))
            .build(),
            children![
                text("Baba Yaga", font_size::TITLE),
                TextColor(color::TEXT_COLOR),
                AnimatedText,
            ]
        )],
    )
}

fn start_screen_body() -> impl Bundle {
    (
        Element::builder(
            node()
                .width(percent(100.0))
                .flex_grow(1.0)
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .build(),
        )
        .build(),
        children![(
            StartScreenButton,
            Button,
            node()
                .width(px(300.0))
                .height(px(80.0))
                .border(px(2.0).all())
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .margin(px(20.0).all())
                .build(),
            
            BorderColor::all(color::GOLD_BORDER),
            BackgroundColor(color::BUTTON_BACKGROUND.with_alpha(0.7)),
            children![(
                text("BEGIN", 48.0),
                TextColor::from(color::TEXT_COLOR_ACTIVE),
            )]
        )],
    )
}

fn start_screen_footer() -> impl Bundle {
    (
        node()
            .width(percent(100.0))
            .height(px(120.0))
            .flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .padding(px(20.0).all())
            .build(),
        BackgroundColor::from(color::BLACK.with_alpha(0.4)),
        children![(
            text(
                "She is a mysterious witch and ogress from Slavic folklore",
                24.0
            ),
            TextColor::from(color::TEXT_COLOR),
        )],
    )
}

// Enhanced button system with more dramatic hover effects
pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<StartScreenButton>),
    >,
    mut game_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = color::BUTTON_BACKGROUND_ACTIVE.into();
                *border_color = color::GOLD_BORDER_ACTIVE.into();
                game_state.set(AppState::AssetLoading);
            }
            Interaction::Hovered => {
                *bg_color = color::BUTTON_BACKGROUND_HOVER.into();
                *border_color = color::GOLD_BORDER_ACTIVE.into();
            }
            Interaction::None => {
                *bg_color = color::BUTTON_BACKGROUND.into();
                *border_color = color::GOLD_BORDER.into();
            }
        }
    }
}

pub fn animate_text(time: Res<Time>, mut query: Query<&mut TextColor, With<AnimatedText>>) {
    for mut color in &mut query {
        let sine = (time.elapsed_secs() * 4.0).sin() * 0.4 + 0.6; // Increased frequency and amplitude
        *color = TextColor::from(Color::srgb(1.0 * sine, 0.5 * sine, 0.3 * sine));
    }
}
