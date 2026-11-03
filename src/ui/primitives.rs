//! Small BSN scenes shared by every screen, menu and the HUD.
//!
//! Each function returns a scene fragment that callers include with `@` and then patch: a
//! fragment sets a sensible base, and the call site layers its own `Node`, colors, observers or
//! `Children` on top.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::Button;

use super::constants::{
    BODY_FONT_SIZE, BRIGHT_GOLD_COLOR, DARK_GRAY_ALPHA_COLOR, DARK_GRAY_COLOR, FOOTER_HEIGHT,
    GOLD_COLOR, GOLD_TEXT_COLOR, HEADER_FONT_SIZE, HEADER_HEIGHT, MUTED_TEXT_COLOR,
    SPLASH_BACKGROUND_COLOR, TAGLINE_TEXT_COLOR, TITLE_FONT_SIZE, TITLE_TEXT_COLOR,
};
use crate::prelude::*;

/// UI text at a fixed pixel size.
pub fn text(message: impl Into<String>, font_size: f32) -> impl Scene {
    let message: String = message.into();
    bsn! {
        Text(message)
        TextFont { font_size: FontSize::Px(font_size) }
    }
}

/// Secondary text, such as a description under a label.
pub fn muted_text(message: impl Into<String>, font_size: f32) -> impl Scene {
    let message: String = message.into();
    bsn! {
        @text(message, font_size)
        TextColor(MUTED_TEXT_COLOR)
    }
}

/// Fixes a node's width. Patch it onto text to line up columns.
pub fn width(width: f32) -> impl Scene {
    bsn! {
        Node { width: px(width) }
    }
}

/// Takes up the free space in a row or column, pushing its siblings apart.
pub fn spacer() -> impl Scene {
    bsn! {
        Node { flex_grow: 1.0 }
    }
}

/// A node that fills the screen as a column and despawns when `state` exits.
pub fn screen<S: States>(state: S) -> impl Scene {
    bsn! {
        ~{DespawnOnExit(state)}
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
        }
    }
}

/// A full-screen menu, drawn above the game and its HUD, opening with a `title` header.
/// Callers add the body and footer as further `Children`.
pub fn menu_screen(menu: Menu, title: &str) -> impl Scene {
    bsn! {
        @screen(menu)
        GlobalZIndex(2)
        Node {
            align_items: AlignItems::Center,
            row_gap: px(20),
        }
        Children [
            @menu_header(title)
        ]
    }
}

/// The dark bar across the top of a menu holding its title.
pub fn menu_header(title: &str) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: HEADER_HEIGHT,
            align_items: AlignItems::Center,
            padding: {UiRect::axes(px(30), px(10))},
        }
        BackgroundColor(DARK_GRAY_COLOR)
        Children [
            @text(title, HEADER_FONT_SIZE)
        ]
    }
}

/// The dark bar across the bottom of a menu. Children are spread across it.
pub fn menu_footer() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: FOOTER_HEIGHT,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: {px(40).horizontal()},
            column_gap: px(20),
        }
        BackgroundColor(DARK_GRAY_COLOR)
    }
}

/// The footer hint shared by every menu.
pub fn unpause_hint() -> impl Scene {
    bsn! {
        @text("Press ESC to unpause", BODY_FONT_SIZE)
    }
}

/// A dark column of rows in the middle of a menu.
pub fn panel(width: f32) -> impl Scene {
    bsn! {
        Node {
            width: px(width),
            flex_direction: FlexDirection::Column,
            padding: px(20),
            row_gap: px(10),
        }
        BackgroundColor(DARK_GRAY_ALPHA_COLOR)
    }
}

/// A thin gold stripe across the screen.
pub fn gold_border() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: px(8),
        }
        BackgroundColor(GOLD_COLOR)
    }
}

/// A gold outline around its children.
pub fn gold_frame() -> impl Scene {
    bsn! {
        Node {
            border: px(2),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BorderColor::all(GOLD_COLOR)
    }
}

/// The layout the start and loading screens share: a pulsing title, a `centerpiece` in the
/// middle, and a `tagline` along the bottom, between two gold stripes.
pub fn splash_screen<S: States>(
    state: S,
    title: &str,
    centerpiece: impl Scene,
    tagline: &str,
) -> impl Scene {
    bsn! {
        @screen(state)
        BackgroundColor(SPLASH_BACKGROUND_COLOR)
        Children [
            @gold_border()
            --
            Node {
                width: percent(100),
                height: px(300),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            Children [
                @gold_frame()
                Node { padding: px(40) }
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3))
                Children [
                    @text(title, TITLE_FONT_SIZE)
                    TextColor(TITLE_TEXT_COLOR)
                    PulsingText
                ]
            ]
            --
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            Children [
                @{centerpiece}
            ]
            --
            Node {
                width: percent(100),
                height: px(120),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: px(20),
            }
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4))
            Children [
                @text(tagline, BODY_FONT_SIZE)
                TextColor(TAGLINE_TEXT_COLOR)
            ]
            --
            @gold_border()
        ]
    }
}

/// Text whose color pulses between dim and bright orange.
#[derive(Component, Default, Clone)]
pub struct PulsingText;

pub(super) fn pulse_text(time: Res<Time>, mut query: Query<&mut TextColor, With<PulsingText>>) {
    let sine = (time.elapsed_secs() * 4.0).sin() * 0.4 + 0.6;
    for mut color in &mut query {
        color.0 = Color::srgb(sine, 0.5 * sine, 0.3 * sine);
    }
}

/// The fill a button shows at rest, under the cursor, and while pressed.
#[derive(Component, Default, Clone, Copy)]
pub struct ButtonColors {
    pub idle: Color,
    pub hovered: Color,
    pub pressed: Color,
}

/// The border a button shows at rest, and while hovered or pressed.
#[derive(Component, Default, Clone, Copy)]
pub struct ButtonBorderColors {
    pub idle: Color,
    pub active: Color,
}

/// Plain gray menu buttons.
pub const MENU_BUTTON_COLORS: ButtonColors = ButtonColors {
    idle: DARK_GRAY_COLOR,
    hovered: Color::srgb(0.15, 0.15, 0.15),
    pressed: Color::srgb(0.25, 0.25, 0.25),
};

/// Warm buttons for the start and game over screens.
pub const GOLD_BUTTON_COLORS: ButtonColors = ButtonColors {
    idle: Color::srgba(0.1, 0.1, 0.1, 0.7),
    hovered: Color::srgba(0.2, 0.15, 0.1, 0.8),
    pressed: Color::srgba(0.3, 0.2, 0.1, 0.9),
};

/// A headless [`Button`] with a centered label that recolors itself as it's hovered and pressed.
/// React to clicks with `on(|_: On<Activate>, ..| ..)` at the call site.
pub fn button(label: &str, font_size: f32, text_color: Color, colors: ButtonColors) -> impl Scene {
    bsn! {
        Button
        Hovered
        colors
        BackgroundColor({colors.idle})
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        Children [
            @text(label, font_size)
            TextColor(text_color)
        ]
    }
}

/// A wide gray button for menus.
pub fn menu_button(label: &str) -> impl Scene {
    bsn! {
        @button(label, 32.0, Color::WHITE, MENU_BUTTON_COLORS)
        Node {
            width: px(300),
            height: px(60),
            border: px(2),
        }
        BorderColor::all(Color::srgb(0.8, 0.8, 0.8))
    }
}

/// A gold-trimmed button for the start and game over screens.
pub fn gold_button(label: &str, font_size: f32) -> impl Scene {
    bsn! {
        @button(label, font_size, GOLD_TEXT_COLOR, GOLD_BUTTON_COLORS)
        Node { border: px(2) }
        BorderColor::all(GOLD_COLOR)
        ButtonBorderColors {
            idle: GOLD_COLOR,
            active: BRIGHT_GOLD_COLOR,
        }
    }
}

/// Recolors every button with [`ButtonColors`] to match its hover and press state.
pub(super) fn style_buttons(
    mut buttons: Query<(
        &Hovered,
        Has<Pressed>,
        &ButtonColors,
        &mut BackgroundColor,
        Option<(&ButtonBorderColors, &mut BorderColor)>,
    )>,
) {
    for (hovered, pressed, colors, mut background, border) in &mut buttons {
        let fill = if pressed {
            colors.pressed
        } else if hovered.get() {
            colors.hovered
        } else {
            colors.idle
        };
        background.set_if_neq(BackgroundColor(fill));

        if let Some((border_colors, mut border)) = border {
            let edge = if pressed || hovered.get() {
                border_colors.active
            } else {
                border_colors.idle
            };
            border.set_if_neq(BorderColor::all(edge));
        }
    }
}
