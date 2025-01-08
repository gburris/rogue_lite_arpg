use bevy::prelude::*;

use crate::despawn::components::Duration;

pub fn remove_expired_entities(
    mut commands: Commands,
    mut duration_query: Query<(Entity, &mut Duration)>,
    time: Res<Time>,
) {
    for (entity, mut duration) in duration_query.iter_mut() {
        duration.0.tick(time.delta());

        if duration.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
