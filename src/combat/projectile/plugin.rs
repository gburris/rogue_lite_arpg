use bevy::prelude::*;

use crate::{
    combat::projectile::{
        events::ProjectileHitEvent, handle_projectile_hit::handle_projectile_hit,
    },
    labels::sets::InGameSet,
};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent>()
            .add_systems(Update, handle_projectile_hit.in_set(InGameSet::Simulation));
    }
}
