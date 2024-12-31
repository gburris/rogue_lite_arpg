use bevy::prelude::*;

use crate::components::BurningEffect;

pub fn process_burning(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BurningEffect)>,
) {
    for (entity, mut burning) in &mut query {
        burning.duration.tick(time.delta());

        // Remove effect when duration expires
        if burning.duration.finished() {
            commands.entity(entity).remove::<BurningEffect>();
        }
    }
}
