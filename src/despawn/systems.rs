use bevy::prelude::*;

use crate::{combat::attributes::Health, despawn::components::LiveDuration};

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

pub fn despawn_on_zero_health(
    mut commands: Commands,
    query: Query<(Entity, &Health), Changed<Health>>,
) {
    for (entity, health) in query.iter() {
        if health.hp == 0.0 && commands.get_entity(entity).is_some() {
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
        commands.entity(e).despawn_recursive();
    }
}
