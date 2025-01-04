use bevy::prelude::*;

use crate::projectile::{components::Projectile, events::DespawnAllProjectiles};

/**
 * For each projectile in world space, we check if time to live has expired and then despawn it
 */
pub fn despawn_long_lived_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Projectile)>,
) {
    for (entity, mut projectile) in query.iter_mut() {
        projectile.time_to_live.tick(time.delta());
        if projectile.time_to_live.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn despawn_all_projectiles(
    _: Trigger<DespawnAllProjectiles>,
    mut commands: Commands,
    mut query: Query<Entity, With<Projectile>>,
) {
    warn!("Depawning all projectiles");
    for entity in query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}
