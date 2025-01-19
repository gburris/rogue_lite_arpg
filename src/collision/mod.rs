mod handle_collisions;

use bevy::prelude::*;

use crate::{collision::handle_collisions::handle_collisions, labels::sets::InGameSet};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_collisions).in_set(InGameSet::Collision));
    }
}
