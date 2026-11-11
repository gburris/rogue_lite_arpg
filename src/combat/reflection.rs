use std::collections::HashSet;

use avian2d::prelude::*;
use bevy::prelude::*;

use super::projectile::{Projectile, ProjectileAttacker};
use crate::prelude::*;

/// Sends projectiles back at its holder's enemies on contact instead of being destroyed by them.
/// It reflects only while it has a collider.
#[derive(Component, Default, Clone)]
#[require(CollidingEntities, Sensor)]
pub struct ProjectileReflection {
    reflected: HashSet<Entity>,
}

/// Reverses each projectile touching a reflector, at most once per projectile, and hands it to
/// the reflector's holder.
pub(super) fn reflect_projectiles(
    mut commands: Commands,
    mut reflector_query: Query<
        (&mut ProjectileReflection, &CollidingEntities, &ChildOf),
        With<Collider>,
    >,
    mut projectile_query: Query<(&mut LinearVelocity, &mut Transform), With<Projectile>>,
    holder_query: Query<&CombatSide>,
) {
    for (mut reflection, colliding_entities, child_of) in &mut reflector_query {
        reflection
            .reflected
            .retain(|&projectile| projectile_query.contains(projectile));
        for &colliding_entity in colliding_entities.iter() {
            if reflection.reflected.contains(&colliding_entity) {
                continue;
            }
            if let Ok((mut linear_velocity, mut transform)) =
                projectile_query.get_mut(colliding_entity)
            {
                // The blocker takes over: the reflected projectile now hurts the blocker's enemies.
                let Ok(side) = holder_query.get(child_of.parent()) else {
                    continue;
                };

                // Reverse direction of projectile! Reflect!
                linear_velocity.0 = -linear_velocity.0;

                // Rotate projectile sprite to face new velocity direction
                transform.rotation = Quat::from_rotation_z(linear_velocity.0.to_angle());

                commands.entity(colliding_entity).insert((
                    CollisionLayers::new(
                        GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
                        LayerMask::from(side.target_hurtbox()) | GameCollisionLayer::HighObstacle,
                    ),
                    ProjectileAttacker {
                        attacker: Some(child_of.parent()),
                    },
                ));

                reflection.reflected.insert(colliding_entity);
            }
        }
    }
}
