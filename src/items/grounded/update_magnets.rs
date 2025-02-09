use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;

use crate::{
    econ::components::{Currency, Wallet},
    items::{Autoloot, Grounded, Magnet},
    player::Player,
};

pub fn update_grounded_magnets(
    time: Res<Time>,
    mut magnet_query: Query<(&mut Transform, &CollidingEntities), (With<Magnet>, With<Grounded>)>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Magnet>, Without<Grounded>)>,
) {
    if let Ok((player_entity, player_transform)) = player_query.get_single() {
        for (mut transform, colliding_entities) in magnet_query.iter_mut() {
            // Only move the item if the player is within the magnet's detection radius
            warn!("Updating magenet");
            if colliding_entities.contains(&player_entity) {
                let direction = (player_transform.translation - transform.translation).normalize();
                transform.translation += direction * 100.0 * time.delta_secs();
            }
        }
    }
}
