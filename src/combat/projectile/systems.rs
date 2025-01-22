use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::combat::{damage::events::DealtDamageEvent, projectile::components::*};

// For certain entities, like projectiles, they have no concept of "health" but instead despawn after "X" hits
pub fn on_damage_dealt_despawn(trigger: Trigger<DealtDamageEvent>, mut commands: Commands) {
    commands.entity(trigger.entity()).despawn_recursive();
}

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

    info!(
        "Spawning projectile w/ direction: {} and velocity: {}",
        aim_direction, velocity
    );

    commands
        .spawn((
            Projectile,
            projectile_bundle.clone(),
            transform,
            LinearVelocity(velocity),
        ))
        .observe(on_damage_dealt_despawn);
}
