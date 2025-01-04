use bevy::prelude::*;

use crate::resources::assets::SpriteAssets;

pub fn setup_ui(mut commands: Commands, sprites: Res<SpriteAssets>) {
    warn!("Setting up UI");
    commands.spawn((
        Transform::from_xyz(20.0, 20.0, 1.0),
        Text::new("(0.0, 0.0)"),
        TextFont::default(),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ));
}
