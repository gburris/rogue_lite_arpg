use bevy::prelude::*;

use crate::{
    damage::{components::DamageEffect, events::DamageEvent},
    projectile::{components::Projectile, events::ProjectileHitEvent},
};

pub fn handle_projectile_hit(
    mut commands: Commands,
    mut collision_events: EventReader<ProjectileHitEvent>,
    projectile_query: Query<&DamageEffect, With<Projectile>>,
) {
    for event in collision_events.read() {
        if let Ok(damage) = projectile_query.get(event.projectile) {
            commands.trigger_targets(
                DamageEvent {
                    damage_source: Some(event.projectile),
                    damage: damage.base_damage,
                    makes_invulnerable: false,
                },
                event.enemy,
            );

            commands.entity(event.projectile).despawn_recursive();
        }
    }
}
