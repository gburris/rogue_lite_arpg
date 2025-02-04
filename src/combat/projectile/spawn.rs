use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    combat::{
        damage::components::DamageSource, projectile::components::*,
        weapon::weapon::ProjectileWeapon,
    },
    configuration::GameCollisionLayer,
};

pub fn spawn_projectile(
    source: DamageSource, //Player, enemy, NPC, Party Member
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
    let collision_target = if source == DamageSource::Enemy {
        GameCollisionLayer::Player
    } else {
        GameCollisionLayer::Enemy
    };
    commands.spawn((
        Projectile,
        weapon.projectile.clone(),
        transform,
        LinearVelocity(velocity),
        AnimationIndices { first: 0, last: 4 },
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        CollisionLayers::new(
            GameCollisionLayer::InAir,
            [collision_target, GameCollisionLayer::HighObstacle],
        ),
    ));
}

fn enemy_projectile_collision_layers() -> CollisionLayers {
    // Currently projectiles can only collide with enemies
    CollisionLayers::new(
        GameCollisionLayer::InAir,
        [GameCollisionLayer::Player, GameCollisionLayer::HighObstacle],
    )
}
