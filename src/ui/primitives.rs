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

pub struct TextBuilder {
    message: String,
    font_size: f32,
    width: Option<f32>,
    pickable: bool,
}

impl TextBuilder {
    pub fn new(message: impl Into<String>, font_size: f32) -> Self {
        Self {
            message: message.into(),
            font_size,
            width: None,
            pickable: true,
        }
    }

    pub fn build(&self) -> impl Bundle {
        (
            Text::new(self.message.clone()),
            TextFont {
                font_size: self.font_size,
                ..default()
            },
            Node {
                width: self.width.map(|w| Val::Px(w)).unwrap_or_default(),
                ..default()
            },
            if self.pickable {
                Pickable::default()
            } else {
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false,
                }
            },
        )
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn not_pickable(mut self) -> Self {
        self.pickable = false;
        self
    }
}

pub fn text(message: impl Into<String>, font_size: f32) -> impl Bundle {
    TextBuilder::new(message, font_size).build()
}
