use bevy::prelude::{Changed, Commands, DespawnRecursiveExt, Entity, Query, Trigger, With};

use crate::{
    components::Health,
    enemy::{events::DespawnAllEnemies, Enemy},
};

//TODO: This should be moved into the "Despawn Domain"
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

pub fn despawn_all_enemies(
    _: Trigger<DespawnAllEnemies>,
    mut commands: Commands,
    mut query: Query<Entity, With<Enemy>>,
) {
    for entity in query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}
