use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;

use crate::{items::Magnet, player::Player};

pub fn update_magnet_locations(
    time: Res<Time>,
    magnet_query: Query<(&Parent, &Magnet, &CollidingEntities)>,
    mut parent_query: Query<&mut Transform, Without<Player>>,
    player_query: Single<(Entity, &Transform), With<Player>>,
) {
    const MAGNETIC_FORCE_CONSTANT: f32 = 10000000.0;
    let (player_entity, player_transform) = player_query.into_inner();
    for (parent_entity, magnet, colliding_entities) in magnet_query.iter() {
        if colliding_entities.contains(&player_entity) {
            if let Ok(mut parent_transform) = parent_query.get_mut(parent_entity.get()) {
                let direction =
                    (player_transform.translation - parent_transform.translation).normalize();
                let distance = player_transform
                    .translation
                    .distance(parent_transform.translation);
                // https://en.wikipedia.org/wiki/Force_between_magnets#Magnetic_dipole_moment
                // https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcTxJQAdhCorNz-fMDq7qdEQhwGPm5YxFYCTQA&s
                let magnetic_force =
                    ((magnet.strength * MAGNETIC_FORCE_CONSTANT) / distance.powi(3)).max(50.0);
                parent_transform.translation += direction * magnetic_force * time.delta_secs();
            }
        }
    }
}
