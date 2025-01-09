use bevy::prelude::{Changed, Commands, DespawnRecursiveExt, Entity, Query};

use crate::components::Health;

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
