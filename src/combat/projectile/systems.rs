use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::{
        damage::{components::CollisionDamage, events::AttemptDamageEvent},
        projectile::components::*,
    },
    enemy::Enemy,
};

pub fn spawn_projectile(
    commands: &mut Commands,
    caster_transform: &Transform,
    caster_aim_position: Vec2,
    projectile_bundle: &ProjectileBundle,
) {
    let spell_speed = 700.0;

    let mut transform = Transform {
        translation: caster_transform.translation,
        ..default()
    };

    let caster_direction = caster_transform.local_x().truncate();
    let aim_direction = caster_aim_position - caster_transform.translation.truncate();
    let angle = caster_direction.angle_to(aim_direction);

    transform.rotate_z(angle);

    let velocity = aim_direction.normalize() * spell_speed;

    trace!("Spawning projectile w/ velocity: {}", velocity);

    commands.spawn((
        Projectile,
        projectile_bundle.clone(),
        transform,
        LinearVelocity(velocity),
    ));
}

pub fn handle_projectile_collisions(
    mut commands: Commands,
    projectile_query: Query<(&CollisionDamage, &CollidingEntities, Entity), With<Projectile>>,
    enemy_query: Query<&Enemy>,
) {
    for (collision_damage, colliding_entities, projectile_entity) in projectile_query.iter() {
        for &colliding_entity in colliding_entities.iter() {
            if enemy_query.contains(colliding_entity) {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage: collision_damage.damage,
                        damage_source: Some(projectile_entity),
                    },
                    colliding_entity,
                );
            }

            // despawn projectile and ignore further collisions after ANY collision
            commands.entity(projectile_entity).despawn_recursive();
            return;
        }
    }
}
