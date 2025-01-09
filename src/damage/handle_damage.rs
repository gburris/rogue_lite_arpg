use bevy::prelude::*;

use crate::{
    components::Health,
    enemy::{
        events::{DamageEvent, EnemyDefeatedEvent},
        Enemy, Experience,
    },
    player::Player,
};

pub fn handle_damage(
    mut damage_events: EventReader<DamageEvent>,
    mut query_set: ParamSet<(
        Query<(Entity, &mut Health, &Transform, &Experience), With<Enemy>>,
        Query<(Entity, &mut Health, &Transform), With<Player>>,
    )>,
    mut enemy_defeated_events: EventWriter<EnemyDefeatedEvent>,
) {
    for event in damage_events.read() {
        // First try to handle as enemy
        let mut handled = false;
        {
            let mut enemy_query = query_set.p0();
            if let Ok((entity, mut health, transform, experience)) =
                enemy_query.get_mut(event.entity)
            {
                if health.hp <= 0.0 {
                    continue; // Skip if already dead
                }
                health.take_damage(event.damage);

                if health.hp == 0.0 {
                    enemy_defeated_events.send(EnemyDefeatedEvent {
                        enemy_entity: entity,
                        enemy_position: transform.translation,
                        exp_value: experience.base_exp,
                    });
                }
                handled = true;
            }
        }

        // If not handled as enemy, try to handle as player
        if !handled {
            let mut player_query = query_set.p1();
            if let Ok((_entity, mut health, _transform)) = player_query.get_mut(event.entity) {
                if health.hp <= 0.0 {
                    continue; // Skip if already dead
                }
                health.take_damage(event.damage);

                if health.hp == 0.0 {
                    warn!("Player is below 0 hp");
                }
            }
        }
    }
}
