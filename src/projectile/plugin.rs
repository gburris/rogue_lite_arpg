use bevy::prelude::*;

use crate::events::ProjectileHitEvent;
use crate::labels::sets::GamePlaySet;
use crate::projectile::handle_projectile_hit;
use crate::projectile::despawn::despawn_projectiles;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent>().add_systems(
            Update,
            (handle_projectile_hit, despawn_projectiles).in_set(GamePlaySet::Simulation),
        );
    }
}
