use crate::player::{components::Player, PlayerExperience, PlayerLevel};
use bevy::prelude::*;

pub fn update(
    player_query: Query<(&Player, &PlayerExperience, &PlayerLevel)>,
    mut text_query: Query<(&mut Text, &mut Transform)>,
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
