use bevy::prelude::*;

use crate::{
    combat::{damage::DamagePlugin, status_effects::plugin::StatusEffectPlugin},
    labels::sets::InGameSet,
};

use super::projectile::systems::on_collision_despawn;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DamagePlugin, StatusEffectPlugin))
            .add_systems(
                Update,
                on_collision_despawn.in_set(InGameSet::DespawnEntities),
            );
    }
}
