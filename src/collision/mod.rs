use bevy::prelude::*;

use crate::labels::sets::GamePlaySet;
pub mod events;
mod handle_collisions;

pub use events::EnemyCollidesWithPlayer;
pub use handle_collisions::handle_collisions;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_collisions).in_set(GamePlaySet::Collision));
    }
}
