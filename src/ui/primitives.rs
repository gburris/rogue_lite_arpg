use bevy::prelude::*;

use crate::ui::element::{Element, node};

use super::constants::{color, font_size, val};

pub fn menu_header(title: &str) -> impl Bundle {
    (
        Element::builder(
            node()
                .width(percent(100.0))
                .height(val::HEADER_HEIGHT)
                .padding(px(30.0).horizontal())
                .build(),
        )
        .background_color(color::DARK_GRAY)
        .build(),
        children![text(title, font_size::HEADER)],
    )
}

pub fn gold_border() -> impl Bundle {
    Element::builder(
        node()
            .width(percent(100.0))
            .height(px(8.0))
            .build(),
    )
    .background_color(BackgroundColor(color::GOLD_BORDER))
    .build()
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
    Element::builder(
        node()
            .width(px(width))
            .build(),
    )
    .build()
}
