use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::projectile::components::*;

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
