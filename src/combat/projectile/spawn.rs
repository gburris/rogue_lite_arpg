use super::components::Projectile;
use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::{
        damage::components::DamageSource, status_effects::components::EffectsList,
        weapon::weapon::ProjectileWeapon,
    },
    configuration::GameCollisionLayer,
    labels::layer::ZLayer,
};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn spawn_projectile_from_weapon(
    damage_source: DamageSource,
    commands: &mut Commands,
    caster_transform: &Transform,
    aim_position: Vec2,
    weapon: &ProjectileWeapon,
) {
    let position = caster_transform.translation;
    let velocity = (aim_position - position.truncate()).normalize() * weapon.projectile_speed;

    spawn_projectile(
        damage_source,
        commands,
        position,
        velocity,
        &weapon.projectile.projectile,
        &weapon.projectile.sprite,
        &weapon.projectile.effects_list,
    );
}

pub fn spawn_projectile(
    damage_source: DamageSource,
    commands: &mut Commands,
    position: Vec3,
    velocity: Vec2,
    projectile: &Projectile,
    sprite: &Sprite,
    effects_list: &EffectsList,
) {
    let mut transform = Transform {
        translation: Vec3::new(position.x, position.y, ZLayer::Projectiles.z()),
        ..default()
    };

    transform.look_to(Vec3::new(velocity.x, velocity.y, 0.0), Vec3::Z);

    let entity = commands
        .spawn((
            projectile.clone(),
            transform,
            LinearVelocity(velocity),
            sprite.clone(),
            effects_list.clone(),
            AnimationIndices {
                first: 0,
                last: 4,
                is_one_shot: false,
            },
            AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
            CollisionLayers::new(
                GameCollisionLayer::InAir,
                LayerMask::from(damage_source) | GameCollisionLayer::HighObstacle,
            ),
        ))
        .id();
}

pub fn spawn_reflected_projectile(
    damage_source: DamageSource,
    commands: &mut Commands,
    original_projectile: &Projectile,
    original_sprite: &Sprite,
    effects_list: &EffectsList,
    impact_position: Vec3,
    incoming_velocity: Vec2,
) {
    if incoming_velocity.length() <= 0.001 {
        error!(
            "CRITICAL ERROR: Incoming velocity is zero or near-zero: {:?}",
            incoming_velocity
        );
        return;
    }

    let reflection_direction = -incoming_velocity.normalize(); // Reverse the velocity
    let reflected_velocity = reflection_direction * incoming_velocity.length();

    spawn_projectile(
        damage_source,
        commands,
        impact_position,
        reflected_velocity,
        original_projectile,
        original_sprite,
        effects_list,
    );
}
