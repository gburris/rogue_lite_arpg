use bevy::{color::Color, ui::Val};

pub mod color {
    use super::*;
    pub const BACKGROUND: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);
    pub const DARK_GRAY: Color = Color::srgb(0.05, 0.05, 0.05);
    pub const DARK_GRAY_ALPHA: Color = Color::srgba(0.05, 0.05, 0.05, 0.98);
    pub const RED: Color = Color::srgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::srgb(0.0, 0.8, 0.0);
    pub const BUTTON_BACKGROUND: Color = Color::srgba(0.15, 0.15, 0.15, 0.7);
    pub const BUTTON_BACKGROUND_ACTIVE: Color = Color::srgba(0.3, 0.2, 0.2, 0.9);
    pub const BUTTON_BACKGROUND_HOVER: Color = Color::srgba(0.2, 0.15, 0.1, 0.8);
    pub const LOAD_SCREEN_BACKGROUND: Color = Color::srgb(0.02, 0.01, 0.04);
    pub const GOLD_BORDER: Color = Color::srgb(0.8, 0.6, 0.2);
    pub const GOLD_BORDER_ACTIVE: Color = Color::srgb(1.0, 0.8, 0.3);
    pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.7, 0.2);
    pub const TEXT_COLOR_ACTIVE: Color = Color::srgb(0.9, 0.8, 0.3);
    pub const TEXT_COLOR_LOADING: Color = Color::srgb(0.7, 0.6, 0.5);
    pub const BLACK: Color = Color::srgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::srgb(1.0, 1.0, 1.0);
}

pub mod val {
    use super::*;
    pub const HEADER_HEIGHT: Val = Val::Px(100.0);
    pub const FOOTER_HEIGHT: Val = Val::Px(80.0);
    pub const HEALTH_TEXT_OFFSET: f32 = 10.0;
}

pub mod font_size {
    pub const TITLE: f32 = 80.0;
    pub const HEADER: f32 = 50.0;
}
