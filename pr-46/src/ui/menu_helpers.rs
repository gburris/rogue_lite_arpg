use bevy::prelude::*;

use super::constants::{DARK_GRAY_COLOR, HEADER_FONT_SIZE, HEADER_HEIGHT};

pub fn spawn_header(builder: &mut ChildBuilder, title: &str) {
    builder
        .spawn((
            Node {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                height: HEADER_HEIGHT,
                padding: UiRect::axes(Val::Px(30.0), Val::Px(10.0)),
                ..default()
            },
            BackgroundColor::from(DARK_GRAY_COLOR),
        ))
        .with_child((
            Text::new(title),
            TextFont {
                font_size: HEADER_FONT_SIZE,
                ..default()
            },
        ));
}
