use crate::{
    components::{Experience, Health},
    enemy::{EnemyDamageEvent, EnemyDefeatedEvent},
};
use bevy::prelude::*;

pub fn handle_enemy_damage(
    mut commands: Commands,
    mut damage_events: EventReader<EnemyDamageEvent>,
    mut query: Query<(Entity, &mut Health, &Transform, &Experience)>,
    mut enemy_defeated_events: EventWriter<EnemyDefeatedEvent>,
) {
    for event in damage_events.read() {
        if let Ok((entity, mut health, transform, experience)) = query.get_mut(event.enemy_entity) {
            if health.hp <= 0.0 {
                //Skip this event, the enemy is already dead!
                //Happens when an enemy takes damage from two sources on the same frame
                continue;
            }
            health.hp -= event.damage;
            if health.hp <= 0.0 {
                commands.entity(entity).despawn();
                enemy_defeated_events.send(EnemyDefeatedEvent {
                    enemy_entity: entity,
                    enemy_position: transform.translation,
                    exp_value: experience.base_exp,
                });
            }
        }
    }
}
