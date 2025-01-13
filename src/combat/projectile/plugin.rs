use bevy::prelude::*;

use crate::{
    combat::projectile::{handle_projectile_hit, projectile_hit_event::ProjectileHitEvent},
    labels::sets::GamePlaySet,
};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent>().add_systems(
            Update,
            handle_projectile_hit.in_set(GamePlaySet::Simulation),
        );
    }
}
