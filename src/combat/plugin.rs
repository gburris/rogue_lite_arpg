use bevy::prelude::*;

use crate::{
    combat::{damage::DamagePlugin, status_effects::plugin::StatusEffectPlugin},
    labels::sets::InGameSet,
};

use super::projectile::systems::handle_projectile_collisions;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DamagePlugin, StatusEffectPlugin))
            .add_systems(
                Update,
                handle_projectile_collisions.in_set(InGameSet::Collision),
            );
    }
}
