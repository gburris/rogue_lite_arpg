use bevy::color::Color;
use bevy::ui::Val;

pub const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);
pub const DARK_GRAY_COLOR: Color = Color::srgb(0.05, 0.05, 0.05);
pub const DARK_GRAY_ALPHA_COLOR: Color = Color::srgba(0.05, 0.05, 0.05, 0.98);
/// Translucent fill behind rows and small buttons inside a panel.
pub const ROW_COLOR: Color = Color::srgba(0.2, 0.2, 0.2, 0.5);
/// Secondary text, such as a stat's description.
pub const MUTED_TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);

/// Near-black backdrop of the start and loading screens.
pub const SPLASH_BACKGROUND_COLOR: Color = Color::srgb(0.02, 0.01, 0.04);
/// Gold trim used for borders on the start and loading screens.
pub const GOLD_COLOR: Color = Color::srgb(0.8, 0.6, 0.2);
/// Brighter gold for a hovered or pressed gold button.
pub const BRIGHT_GOLD_COLOR: Color = Color::srgb(1.0, 0.8, 0.3);
pub const GOLD_TEXT_COLOR: Color = Color::srgb(0.9, 0.8, 0.3);
pub const TITLE_TEXT_COLOR: Color = Color::srgb(0.9, 0.7, 0.2);
pub const TAGLINE_TEXT_COLOR: Color = Color::srgb(0.7, 0.6, 0.5);

pub const HEADER_HEIGHT: Val = Val::Px(100.0);
pub const FOOTER_HEIGHT: Val = Val::Px(80.0);

pub const TITLE_FONT_SIZE: f32 = 80.0;
pub const HEADER_FONT_SIZE: f32 = 50.0;
/// Size of footer text and most labels.
pub const BODY_FONT_SIZE: f32 = 24.0;
/// Size of column headings and inventory rows.
pub const SMALL_FONT_SIZE: f32 = 18.0;
/// Size of a stat's description under its name.
pub const CAPTION_FONT_SIZE: f32 = 16.0;
