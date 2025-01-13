use bevy::prelude::*;

use crate::{collision::EnemyCollidesWithPlayer, damage::events::DamageEvent, player::Player};

// System to handle enemy collision and damage
pub fn handle_enemy_collision(
    collision_event: Trigger<EnemyCollidesWithPlayer>,
    mut commands: Commands,
    mut player_query: Query<Entity, With<Player>>,
) {
    warn!("EnemyCollidesWithPlayer Observer triggered");
    if let Ok(player_entity) = player_query.get_single_mut() {
        warn!("Player damage system start");

        commands.trigger_targets(
            DamageEvent {
                damage: collision_event.collision_damage.damage,
                // damage source is the enemy
                damage_source: Some(collision_event.entity()),
                makes_invulnerable: true,
            },
            player_entity, // target of damage is player
        );
    }
}
