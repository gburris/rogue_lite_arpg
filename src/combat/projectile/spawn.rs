use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::{damage::DamageSource, projectile::projectile_weapon::ProjectileWeapon},
    configuration::{GameCollisionLayer, ZLayer},
};

const PROJECTILE_SPAWN_OFFSET: f32 = 25.0;

pub fn spawn_projectile(
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
        AnimationIndices {
            first: 0,
            last: 4,
            is_one_shot: false,
        },
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        CollisionLayers::new(
            GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
            LayerMask::from(damage_source) | GameCollisionLayer::HighObstacle,
        ),
    ));
}
