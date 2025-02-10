use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;

use crate::{
    items::{Grounded, Magnet},
    player::Player,
};

pub fn update_grounded_magnets(
    time: Res<Time>,
    magnet_query: Query<(&Parent, &Magnet, &CollidingEntities), With<Magnet>>,
    mut parent_query: Query<&mut Transform, (Without<Magnet>, With<Grounded>)>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Magnet>, Without<Grounded>)>,
) {
    if let Ok((player_entity, player_transform)) = player_query.get_single() {
        for (parent_entity, magnet, colliding_entities) in magnet_query.iter() {
            if colliding_entities.contains(&player_entity) {
                if let Ok(mut parent_transform) = parent_query.get_mut(parent_entity.get()) {
                    let direction =
                        (player_transform.translation - parent_transform.translation).normalize();
                    let distance = player_transform
                        .translation
                        .distance(parent_transform.translation);

                    // Calculate speed based on magnet strength and distance
                    let speed = magnet.strength / distance;

                    parent_transform.translation += direction * speed * time.delta_secs();
                }
            }
        }
    }
}
