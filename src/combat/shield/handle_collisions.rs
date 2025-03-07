use crate::{
    combat::{
        damage::components::DamageSource,
        projectile::{components::*, spawn_projectile},
        shield::components::ProjectileReflection,
        status_effects::components::EffectsList,
        weapon::weapon::ProjectileWeapon,
    },
    items::equipment::Equipped,
};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn handle_projectile_reflection_collisions(
    mut commands: Commands,
    projectile_reflector_query: Query<(
        &ProjectileReflection,
        &CollidingEntities,
        Entity,
        &Transform,
    )>,
    projectile_query: Query<(&Projectile, &Transform, &LinearVelocity, Entity)>,
    equipped_query: Query<&Equipped>,
    weapon_query: Query<&ProjectileWeapon>,
    world: &World, // Use the world reference to manually check components
) {
    for (_, colliding_entities, reflector_entity, reflector_transform) in
        projectile_reflector_query.iter()
    {
        // Get the shield owner
        let shield_owner = match equipped_query.get(reflector_entity) {
            Ok(equipped) => equipped.get_equipped_to(),
            Err(_) => continue, // Shield is not equipped, skip
        };

        for &colliding_entity in colliding_entities.iter() {
            // If the entity has all necessary components, process reflection
            if let Ok((projectile, projectile_transform, linear_velocity, _)) =
                projectile_query.get(colliding_entity)
            {
                info!("Reflecting projectile with shield!");

                // Create a temporary weapon structure for the reflection
                //I don't like this method at all
                let reflection_weapon = ProjectileWeapon {
                    projectile: ProjectileBundle {
                        projectile: projectile.clone(),
                        sprite: Sprite::default(), // This will be cloned from the original in spawn_projectile
                        effects_list: EffectsList::default(), // Assuming EffectsList has a default or get from original
                    },
                    projectile_speed: linear_velocity.0.length(), // Use the original projectile's speed
                    spread: 0.0,                                  // No spread for the reflection
                };

                // Calculate the reflection direction
                let reflection_point = reflector_transform.translation.truncate();
                let incoming_direction = linear_velocity.0.normalize();
                let reflection_aim_position = reflection_point - incoming_direction * 100.0;

                // Create new damage source from shield owner
                let new_damage_source = DamageSource::Player;

                // Spawn the reflected projectile using the existing function
                warn!("Spawning reflected projectile!");
                spawn_projectile(
                    new_damage_source,
                    &mut commands,
                    reflector_transform, // Use the shield's transform as the origin
                    reflection_aim_position, // Aim in the opposite direction
                    &reflection_weapon,
                );

                // Despawn the original projectile
                commands.entity(colliding_entity).despawn_recursive();
            }
        }
    }
}
