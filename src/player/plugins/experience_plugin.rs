use bevy::prelude::*;

use crate::{
    enemy::EnemyDefeatedEvent,
    labels::sets::GamePlaySet,
    player::{animate_level_up, handle_player_level_up, PlayerLevelUpEvent},
    enemy::handle_enemy_defeated,
};

pub struct ExperiencePlugin;

impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyDefeatedEvent>()
            .add_event::<PlayerLevelUpEvent>()
            .add_systems(
                Update,
                (
                    handle_enemy_defeated,
                    handle_player_level_up,
                    animate_level_up,
                )
                    .in_set(GamePlaySet::Simulation),
            );
    }
}
