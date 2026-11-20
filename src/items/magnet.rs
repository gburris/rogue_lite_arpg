use avian2d::prelude::*;
use bevy::prelude::*;

use crate::prelude::*;

const MAGNETIC_FORCE: f32 = 2_000_000.0;

#[derive(Component, Default, Clone)]
#[require(
    CollidingEntities,
    Sensor,
    Collider::circle(200.0),
     CollisionLayers::new(
        GameCollisionLayer::Interaction,
        [GameCollisionLayer::PlayerInteractionRadius]
    )
)]
pub struct Magnet;

pub(super) fn update_magnet_locations(
    magnet_query: Query<(&ChildOf, &GlobalTransform, &CollidingEntities), With<Magnet>>,
    player_query: Single<(Entity, &GlobalTransform), With<PlayerInteractionRadius>>,
    mut forces: Query<Forces>,
) {
    let (player_entity, player_transform) = player_query.into_inner();

    for (child_of, magnet_transform, colliding_entities) in &magnet_query {
        if colliding_entities.contains(&player_entity) {
            let magnet_to_player =
                player_transform.translation().xy() - magnet_transform.translation().xy();
            // Touching the player leaves no direction to pull along.
            let Some(direction) = magnet_to_player.try_normalize() else {
                continue;
            };

            // Measured in tiles so the distance isn't too large when cubed.
            let distance = magnet_to_player.length() / TILE_SIZE as f32;

            // https://en.wikipedia.org/wiki/Force_between_magnets#Magnetic_dipole_moment
            // Force gets stronger as magnet gets closer to ensure it closes in on target
            let magnetic_force = MAGNETIC_FORCE / distance.powi(3);

            trace!(
                "Magnetic force applied: {} from distance: {}",
                magnetic_force, distance
            );

            // Apply a new force each tick of fixed update, erasing previous force (persistence =
            // false)
            forces
                .get_mut(child_of.parent())
                .expect("Magnet parent missing or has no forces")
                .apply_force(direction * magnetic_force);
        }
    }
}
