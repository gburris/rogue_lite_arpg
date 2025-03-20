use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::{damage::DamageSource, projectile::projectile_weapon::ProjectileWeapon},
    configuration::GameCollisionLayer,
};

pub fn spawn_projectile(
    damage_source: DamageSource, //Player, enemy, NPC, Party Member
    commands: &mut Commands,
    caster_transform: &Transform,
    caster_aim_position: Vec2,
    weapon: &ProjectileWeapon,
) {
    let mut transform = Transform {
        translation: caster_transform.translation,
        ..default()
    };

    let caster_direction = caster_transform.local_x().truncate();
    let aim_direction = caster_aim_position - caster_transform.translation.truncate();
    let angle = caster_direction.angle_to(aim_direction);

    transform.rotate_z(angle);

    let velocity = aim_direction.normalize() * weapon.projectile_speed;

    trace!("Spawning projectile w/ velocity: {}", velocity);

    commands.spawn((
        weapon.projectile.clone(),
        transform,
        LinearVelocity(velocity),
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
    ));
}
