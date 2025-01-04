use bevy::prelude::*;

use crate::{
    labels::sets::GamePlaySet,
    projectile::{
        despawn::{despawn_all_projectiles, despawn_long_lived_projectiles},
        handle_projectile_hit,
        projectile_hit_event::ProjectileHitEvent,
    },
};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent>()
            .add_observer(despawn_all_projectiles)
            .add_systems(
                Update,
                (handle_projectile_hit, despawn_long_lived_projectiles)
                    .in_set(GamePlaySet::Simulation),
            );
    }
}
