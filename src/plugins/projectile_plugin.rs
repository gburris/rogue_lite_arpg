use bevy::prelude::*;

use crate::events::ProjectileHitEvent;
use crate::systems::check_projectile_collision;
use crate::systems::check_warpzone_collision;
use crate::systems::despawn::despawn_projectiles;
use crate::systems::handle_projectile_collision;
use crate::systems::handle_status_effects;
use crate::systems::move_projectiles;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent>().add_systems(
            Update,
            (
                move_projectiles,
                check_projectile_collision,
                check_warpzone_collision,
                handle_projectile_collision,
                handle_status_effects,
                despawn_projectiles,
            ),
        );
    }
}
