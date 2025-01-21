use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::projectile::{
    components::ProjectileBundle, on_damage_dealt::on_damage_dealt_despawn,
};

pub struct SpellFactory;

impl SpellFactory {
    pub fn spawn_spell(
        commands: &mut Commands,
        caster_transform: &Transform,
        caster_aim_position: Vec2,
        projectile_bundle: &ProjectileBundle,
    ) {
        let spell_speed = 300.0;

        let mut transform = Transform {
            translation: caster_transform.translation,
            ..default()
        };

        let caster_direction = caster_transform.local_x().truncate();
        let aim_direction = caster_aim_position - caster_transform.translation.truncate();
        let angle = caster_direction.angle_to(aim_direction);

        transform.rotate_z(angle);

        let velocity = aim_direction.normalize() * spell_speed;

        trace!("Spawning projectile w/ direction: {}", aim_direction);

        commands
            .spawn((
                projectile_bundle.clone(),
                transform,
                LinearVelocity(velocity),
            ))
            .observe(on_damage_dealt_despawn);
    }
}
