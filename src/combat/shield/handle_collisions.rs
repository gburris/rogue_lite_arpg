use crate::{
    combat::{damage::DamageSource, shield::components::ProjectileReflection, Projectile},
    configuration::GameCollisionLayer,
    prelude::Enemy,
};
use avian2d::prelude::*;
use bevy::prelude::*;

use super::ActiveShield;

pub fn handle_projectile_reflection_collisions(
    mut shield_query: Query<
        (&mut ActiveShield, &CollidingEntities, &ChildOf),
        With<ProjectileReflection>,
    >,
    mut projectile_query: Query<
        (&mut LinearVelocity, &mut CollisionLayers, &mut Transform),
        With<Projectile>,
    >,
    enemy_query: Query<&Enemy>,
) {
    for (mut shield, colliding_entities, child_of) in shield_query.iter_mut() {
        for &colliding_entity in colliding_entities.iter() {
            if shield.projectiles_reflected.contains(&colliding_entity) {
                continue;
            }
            if let Ok((mut linear_velocity, mut collision_layers, mut transform)) =
                projectile_query.get_mut(colliding_entity)
            {
                // If holder is enemy and it is reflected, it can now hurt the player!
                let new_damage_source = if enemy_query.contains(child_of.parent) {
                    DamageSource::Enemy
                } else {
                    DamageSource::Player
                };

                // Reverse direction of projectile! Reflect!
                linear_velocity.0 = -linear_velocity.0;

                // Rotate projectile sprite to face new velocity direction
                transform.rotation = Quat::from_rotation_z(linear_velocity.0.to_angle());

                *collision_layers = CollisionLayers::new(
                    GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
                    LayerMask::from(new_damage_source) | GameCollisionLayer::HighObstacle,
                );
                shield.projectiles_reflected.insert(colliding_entity);
            }
        }
    }
}
