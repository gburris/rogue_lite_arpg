use avian2d::prelude::{CollidingEntities, ExternalForce};
use bevy::prelude::*;

use crate::{items::Magnet, player::interact::PlayerInteractionRadius};

const MAGNETIC_FORCE: f32 = 2000000.0;

pub fn update_magnet_locations(
    mut commands: Commands,
    magnet_query: Query<(&ChildOf, &GlobalTransform, &CollidingEntities), With<Magnet>>,
    player_query: Query<(Entity, &GlobalTransform), With<PlayerInteractionRadius>>,
) {
    let Ok((player_entity, player_transform)) = player_query.get_single() else {
        return;
    };

    for (child_of, magnet_transform, colliding_entities) in magnet_query.iter() {
        if colliding_entities.contains(&player_entity) {
            let direction = (player_transform.translation().truncate()
                - magnet_transform.translation().truncate())
            .normalize();

            // divide distance by 32 (tile size) so it isn't too large when used in cubic function
            let distance = player_transform
                .translation()
                .distance(magnet_transform.translation())
                / 32.0;

            // https://en.wikipedia.org/wiki/Force_between_magnets#Magnetic_dipole_moment
            // https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcTxJQAdhCorNz-fMDq7qdEQhwGPm5YxFYCTQA&s
            // Force gets stronger as magnet gets closer to ensure it closes in on target
            let magnetic_force = MAGNETIC_FORCE / distance.powi(3);

            trace!(
                "Magnetic force applied: {} from distance: {}",
                magnetic_force,
                distance
            );

            // Apply a new force each tick of fixed update, erasing previous force (persistence = false)
            commands
                .entity(child_of.parent)
                .insert(ExternalForce::new(direction * magnetic_force).with_persistence(false));
        }
    }
}
