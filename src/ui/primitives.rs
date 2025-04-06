use bevy::prelude::*;

use super::constants::{DARK_GRAY_COLOR, HEADER_FONT_SIZE, HEADER_HEIGHT};

pub fn menu_header(title: &str) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            height: HEADER_HEIGHT,
            padding: UiRect::axes(Val::Px(30.0), Val::Px(10.0)),
            ..default()
        },
        BackgroundColor::from(DARK_GRAY_COLOR),
        Children::spawn_one((
            Text::new(title),
            TextFont {
                font_size: HEADER_FONT_SIZE,
                ..default()
            },
        )),
    )
}

pub fn gold_border() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(8.0),
            ..default()
        },
        BackgroundColor::from(Color::srgb(0.8, 0.6, 0.2)),
    )
}

pub fn text(message: impl Into<String>, font_size: f32) -> impl Bundle {
    (
        Text::new(message),
        TextFont {
            font_size,
            ..default()
        },
    )
}

pub fn width(width: f32) -> impl Bundle {
    Node {
        width: Val::Px(width),
        ..default()
    }
}
