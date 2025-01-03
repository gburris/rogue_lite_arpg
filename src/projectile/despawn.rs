use bevy::prelude::*;

use super::Projectile;

/**
 * For each projectile in world space, we check if time to live has expired and then despawn it
 */
pub fn despawn_projectiles(
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
