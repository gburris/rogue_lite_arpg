use bevy::prelude::*;

use crate::{
    components::DamageEffect,
    enemy::events::EnemyDamageEvent,
    projectile::events::ProjectileHitEvent,
    status_effects::{components::EffectsList, events::ApplyEffect},
};

pub fn handle_projectile_hit(
    mut commands: Commands,
    mut collision_events: EventReader<ProjectileHitEvent>,
    mut enemy_damaged_events: EventWriter<EnemyDamageEvent>,
    projectile_query: Query<(&DamageEffect, &EffectsList)>,
) {
    for event in collision_events.read() {
        if let Ok((damage, effects_list)) = projectile_query.get(event.projectile) {
            // We apply statuses first, then raw damage to avoid scenario where damage
            // kills entity before we try to add statuses it (causes panic)
            commands.trigger_targets(
                ApplyEffect {
                    effect: effects_list.effects.clone(),
                },
                event.enemy,
            );

            enemy_damaged_events.send(EnemyDamageEvent {
                enemy_entity: event.enemy,
                damage_source: Some(event.projectile),
                damage: damage.base_damage,
            });

            commands.entity(event.projectile).despawn_recursive();
        }
    }
}
