use bevy::prelude::*;

use crate::despawn::components::LiveDuration;

pub fn despawn_expired_entities(
    mut commands: Commands,
    mut duration_query: Query<(Entity, &mut LiveDuration)>,
    time: Res<Time>,
) {
    for (entity, mut duration) in duration_query.iter_mut() {
        duration.0.tick(time.delta());

        if duration.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/**
 * Despawn all entities with the specific component
 */
pub fn despawn_all<T: Event, C: Component>(
    _: Trigger<T>,
    mut commands: Commands,
    query: Query<Entity, With<C>>,
) {
    for e in query.iter() {
        // debug!("Despawning entity: {}", e);
        commands.entity(e).despawn_recursive();
    }
}

/**
 * Despawn singleton entity with the specific component
 */
pub fn despawn_single<C: Component>(mut commands: Commands, entity: Single<Entity, With<C>>) {
    // debug!("Despawning single entity: {}", *entity);
    commands.entity(*entity).despawn_recursive();
}
