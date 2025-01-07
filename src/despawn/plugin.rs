use bevy::prelude::*;

use crate::{despawn::systems::*, labels::sets::GamePlaySet};
pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            remove_expired_entities.in_set(GamePlaySet::DespawnEntities),
        );
    }
}
