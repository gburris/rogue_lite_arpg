use crate::{
    combat::{
        damage::components::DamageSource,
        projectile::{components::*, spawn::spawn_reflected_projectile, spawn_projectile},
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
    projectile_query: Query<(
        &Projectile,
        &Transform,
        &LinearVelocity,
        &Sprite,
        &EffectsList,
        Entity,
    )>,
    equipped_query: Query<&Equipped>,
    // Add parent transform query to get the parent/caster's position
    parent_query: Query<&Transform>,
) {
    for (_, colliding_entities, reflector_entity, reflector_transform) in
        projectile_reflector_query.iter()
    {
        let shield_owner = match equipped_query.get(reflector_entity) {
            Ok(equipped) => equipped.get_equipped_to(),
            Err(_) => continue, // Skip if not equipped
        };

        // Get the caster/owner transform
        let caster_transform = match parent_query.get(shield_owner) {
            Ok(transform) => transform,
            Err(_) => {
                warn!(
                    "Could not find caster transform for shield owner: {:?}",
                    shield_owner
                );
                continue; // Skip if we can't get the caster transform
            }
        };

        info!("Shield owner position: {:?}", caster_transform.translation);
        info!("Shield position: {:?}", reflector_transform.translation);

        for &colliding_entity in colliding_entities.iter() {
            if let Ok((
                projectile,
                projectile_transform,
                linear_velocity,
                sprite,
                effects_list,
                _,
            )) = projectile_query.get(colliding_entity)
            {
                info!("Reflecting projectile!");

                // Pass both shield position and caster position to the reflection function
                let impact_position =
                    reflector_transform.translation + caster_transform.translation;
                let incoming_velocity = linear_velocity.0;
                let new_damage_source = DamageSource::Player;

                spawn_reflected_projectile(
                    new_damage_source,
                    &mut commands,
                    projectile,
                    sprite,
                    effects_list,
                    impact_position,
                    incoming_velocity,
                );

                commands.entity(colliding_entity).despawn_recursive();
            }
        }
    }
}
