use bevy::prelude::*;

use super::{DespawnAllNPCs, NPC};

pub fn despawn_all_npcs(
    _: Trigger<DespawnAllNPCs>,
    mut commands: Commands,
    mut tile_query: Query<Entity, With<NPC>>,
) {
    debug!("Depawning all NPCs");
    for entity in tile_query.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}
