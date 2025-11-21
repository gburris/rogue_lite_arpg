use bevy::prelude::*;

use crate::{
    prelude::AppState,
    ui::{
        constants::{color, font_size},
        element::{Element, node},
        primitives::{gold_border, text},
    },
};

#[derive(Component)]
pub struct LoadScreen;

#[derive(Component)]
pub struct AnimatedText;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        LoadScreen,
        DespawnOnExit(AppState::SpawnZone),
        Element::builder(
            node()
                .width(percent(100.0))
                .height(percent(100.0))
                .flex_direction(FlexDirection::Column)
                // Make sure there's no extra space on sides
                .padding(px(0.0).all())
                .margin(px(0.0).all())
                .build(),
        )
        .background_color(color::LOAD_SCREEN_BACKGROUND)
        .global_z_index(1)
        .build(),
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
        node()
            .width(percent(100.0))
            .height(px(300.0))
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .margin(px(0.0).all())
            .padding(px(0.0).all())
            .build(),
        children![(
            node()
                .width(auto())
                .height(auto())
                .border(px(2.0).all())
                .padding(px(40.0).horizontal())
                .margin(px(0.0).all())
                .build(),
            BorderColor::all(color::GOLD_BORDER),
            BackgroundColor::from(color::BLACK.with_alpha(0.3)),
            children![(
                text("Loading Instance", font_size::TITLE),
                TextColor::from(color::TEXT_COLOR),
                AnimatedText,
            )]
        )],
    )
}

fn body_section() -> impl Bundle {
    (
        node()
            .width(percent(100.0))
            .flex_grow(1.0)
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .margin(px(0.0).all())
            .padding(px(0.0).all())
            .build(),
        children![(
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
                text("Loading Instance", 48.0),
                TextColor::from(color::TEXT_COLOR_ACTIVE),
            )]
        )],
    )
}

fn footer_section() -> impl Bundle {
    (
        node()
            .width(percent(100.0))
            .height(px(120.0))
            .flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .padding(px(20.0).all())
            .margin(px(0.0).all())
            .build(),
        BackgroundColor::from(color::BLACK.with_alpha(0.4)),
        children![(
            text("I'm loading", 24.0),
            TextColor::from(color::TEXT_COLOR_LOADING),
        )],
    )
}

pub fn animate_text(time: Res<Time>, mut query: Query<&mut TextColor, With<AnimatedText>>) {
    for mut color in &mut query {
        let sine = (time.elapsed_secs() * 4.0).sin() * 0.4 + 0.6; // Increased frequency and amplitude
        *color = TextColor::from(Color::srgb(1.0 * sine, 0.5 * sine, 0.3 * sine));
    }
}
