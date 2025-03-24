use crate::{
    combat::{
        damage::DamageSource, projectile::components::*, shield::components::ProjectileReflection,
    },
    configuration::GameCollisionLayer,
    enemy::Enemy,
};
use avian2d::prelude::*;
use bevy::prelude::*;

use super::ActiveShield;

pub fn handle_projectile_reflection_collisions(
    mut projectile_reflector_query: Query<(
        &ProjectileReflection,
        &mut ActiveShield,
        &CollidingEntities,
        &Parent,
    )>,
    mut projectile_query: Query<(
        &Projectile,
        &mut LinearVelocity,
        &mut CollisionLayers,
        &mut Transform,
    )>,
    enemy_query: Query<&Enemy>,
) {
    for (_, mut shield, colliding_entities, holder) in projectile_reflector_query.iter_mut() {
        for &colliding_entity in colliding_entities.iter() {
            if shield.projectiles_reflected.contains(&colliding_entity) {
                continue;
            }
            if let Ok((_, mut linear_velocity, mut collision_layers, mut transform)) =
                projectile_query.get_mut(colliding_entity)
            {
                let incoming_velocity = linear_velocity.0;
                let mut new_damage_source = DamageSource::Player;
                if let Ok(_enemy) = enemy_query.get(holder.get()) {
                    new_damage_source = DamageSource::Enemy;
                }
                let reflection_direction = -incoming_velocity.normalize();
                let reflected_velocity = reflection_direction * incoming_velocity.length();
                linear_velocity.0 = reflected_velocity;
                let angle = reflected_velocity.y.atan2(reflected_velocity.x);
                transform.rotation = Quat::from_rotation_z(angle);
                *collision_layers = CollisionLayers::new(
                    GameCollisionLayer::InAir,
                    LayerMask::from(new_damage_source) | GameCollisionLayer::HighObstacle,
                );
                shield.projectiles_reflected.insert(colliding_entity);
            }
        }
    }
}
