use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    configuration::{GameCollisionLayer, ZLayer},
    utility::Lifespan,
};

use super::{
    damage::{AttemptDamageEvent, Damage, DamageSource, HurtBox},
    shield::components::ProjectileReflection,
    status_effects::components::EffectsList,
};

#[derive(Component, Clone)]
#[require(
    Lifespan::new(1.0),
    Sensor,
    RigidBody,
    Collider::rectangle(10.0, 10.0),
    CollidingEntities
)]
pub struct Projectile {
    pub damage: (f32, f32),
}

#[derive(Bundle, Clone)]
pub struct ProjectileBundle {
    pub projectile: Projectile,
    pub sprite: Sprite,
    pub effects_list: EffectsList,
}

#[derive(Component)]
pub struct ProjectileWeapon {
    pub projectile: ProjectileBundle,
    pub projectile_speed: f32,
    pub spread: f32,
}

const PROJECTILE_SPAWN_OFFSET: f32 = 25.0;

pub fn spawn(
    damage_source: DamageSource, //Player, enemy, NPC, Party Member
    commands: &mut Commands,
    caster_transform: &Transform,
    caster_aim_position: Vec2,
    weapon: &ProjectileWeapon,
) {
    let caster_direction = caster_transform.local_x().truncate();
    let aim_direction = (caster_aim_position - caster_transform.translation.truncate()).normalize();
    let angle = caster_direction.angle_to(aim_direction);

    let velocity = aim_direction * weapon.projectile_speed;

    let starting_positon =
        caster_transform.translation.truncate() + (PROJECTILE_SPAWN_OFFSET * aim_direction);

    trace!("Spawning projectile w/ velocity: {}", velocity);

    commands.spawn((
        weapon.projectile.clone(),
        Transform {
            translation: starting_positon.extend(ZLayer::InAir.z()),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        LinearVelocity(velocity),
        AnimationIndices::Cycle((0..=4).cycle()),
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        CollisionLayers::new(
            GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
            LayerMask::from(damage_source) | GameCollisionLayer::HighObstacle,
        ),
    ));
}

pub fn handle_collisions(
    mut commands: Commands,
    projectile_query: Query<(&Projectile, &CollidingEntities, Entity)>,
    hurt_box_query: Query<&HurtBox>,
    reflector_query: Query<&ProjectileReflection>,
) {
    for (projectile, colliding_entities, projectile_entity) in projectile_query.iter() {
        // ignore further collisions after ANY collision with the projectile
        if let Some(&colliding_entity) = colliding_entities.iter().next() {
            // If the thing we collide with has a HurtBox, lets try to damage it!
            if hurt_box_query.contains(colliding_entity) {
                commands.trigger_targets(
                    AttemptDamageEvent {
                        ignore_invulnerable: false,
                        damage: Damage::Range(projectile.damage),
                        damage_source: Some(projectile_entity),
                    },
                    colliding_entity,
                );
            }
            if reflector_query.contains(colliding_entity) {
                continue;
            }
            commands.entity(projectile_entity).despawn();
        }
    }
}
