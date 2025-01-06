use crate::player::{components::Player, PlayerExperience, PlayerLevel};
use bevy::prelude::*;

#[derive(Component)]
pub struct GameOverlay;

pub fn create(mut commands: Commands) {
    debug!("Setting up game overlay UI");
    commands.spawn((
        GameOverlay,
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

pub fn update(
    player_query: Query<(&Player, &PlayerExperience, &PlayerLevel)>,
    mut text_query: Query<(&mut Text, &mut Transform), With<GameOverlay>>,
) {
    for (_player, exp, level) in player_query.iter() {
        for (mut text, mut transform) in text_query.iter_mut() {
            *text = Text::new(format!(
                "Level: {:.1} Exp: {:.1} / {:.1}",
                level.current, exp.current, exp.next_level_requirement
            ));

            transform.translation = Vec3::new(20.0, 20.0, 1.0);
        }
    }
}
