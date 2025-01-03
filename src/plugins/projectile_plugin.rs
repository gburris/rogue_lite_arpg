use bevy::prelude::*;

use crate::events::ProjectileHitEvent;
use crate::labels::sets::GamePlaySet;
use crate::systems::check_projectile_collision;
use crate::systems::check_warpzone_collision;
use crate::systems::despawn::despawn_projectiles;
use crate::systems::handle_projectile_collision;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent>().add_systems(
            Update,
            (
                check_projectile_collision,
                check_warpzone_collision,
                handle_projectile_collision,
                despawn_projectiles,
            )
                .in_set(GamePlaySet::Simulation),
        );
    }
}
