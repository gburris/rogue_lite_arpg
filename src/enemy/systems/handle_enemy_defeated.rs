use bevy::prelude::*;

use crate::enemy::EnemyDefeatedEvent;
use crate::player::components::Player;
use crate::player::components::PlayerExperience;
use crate::player::components::PlayerLevel;
use crate::player::PlayerLevelUpEvent;

pub fn handle_enemy_defeated(
    mut enemy_defeat_events: EventReader<EnemyDefeatedEvent>,
    mut player_query: Query<(&mut PlayerExperience, &mut PlayerLevel, &Transform), With<Player>>,
    mut level_up_events: EventWriter<PlayerLevelUpEvent>,
) {
    for event in enemy_defeat_events.read() {
        if let Ok((mut exp, mut level, transform)) = player_query.get_single_mut() {
            exp.current += event.exp_value;
            // Check for level up
            while exp.current >= exp.next_level_requirement {
                level.current += 1;
                exp.current -= exp.next_level_requirement;
                exp.next_level_requirement *= 2; // Double exp requirement

                level_up_events.send(PlayerLevelUpEvent {
                    new_level: level.current,
                    position: transform.translation,
                });
            }
        }
    }
}
