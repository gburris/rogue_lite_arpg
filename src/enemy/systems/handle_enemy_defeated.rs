use bevy::prelude::*;

use crate::{
    combat::damage::events::DefeatedEvent,
    enemy::{Enemy, Experience},
    player::components::{Player, PlayerExperience},
};

pub fn on_enemy_defeated(
    trigger: Trigger<DefeatedEvent>,
    defeated_enemy_query: Query<&Experience, With<Enemy>>,
    mut player_query: Query<&mut PlayerExperience, With<Player>>,
) {
    if let Ok(experience_to_gain) = defeated_enemy_query.get(trigger.entity()) {
        if let Ok(mut experience) = player_query.get_single_mut() {
            // Increase player experience on enemy defeat
            experience.current += experience_to_gain.base_exp;
        }
    }

    // TODO: add enemy defeated animation
}
