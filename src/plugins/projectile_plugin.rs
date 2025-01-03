use bevy::prelude::*;

use crate::events::ProjectileHitEvent;
use crate::labels::sets::GamePlaySet;
use crate::systems::despawn::despawn_projectiles;
use crate::systems::handle_projectile_collision;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent>().add_systems(
            Update,
            (handle_projectile_collision, despawn_projectiles).in_set(GamePlaySet::Simulation),
        );
    }
}
