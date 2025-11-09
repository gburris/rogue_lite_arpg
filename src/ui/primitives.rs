use bevy::prelude::*;

use crate::ui::element::Element;

use super::constants::{color, font_size, val};

pub fn menu_header(title: &str) -> impl Bundle {
    (
        Element::new()
            .width(percent(100.0))
            .height(val::HEADER_HEIGHT)
            .padding(px(30.0).horizontal())
            .background_color(color::DARK_GRAY),
        children![text(title, font_size::HEADER)],
    )
}

pub fn gold_border() -> impl Bundle {
    Element::new()
        .width(percent(100.0))
        .height(px(8.0))
        .background_color(color::GOLD_BORDER)
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
    Element::new().width(px(width))
}
