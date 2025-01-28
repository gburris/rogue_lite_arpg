use crate::combat::weapon::weapon::Weapon;

use super::components::{MeleeAttack, MeleeSwingMarker, MeleeSwingType};
use bevy::prelude::*;

pub fn update_melee_attacks(
    time: Res<Time>,
    mut commands: Commands,
    caster_query: Query<&Transform, Without<MeleeSwingMarker>>,
    mut melee_query: Query<
        (
            Entity,
            &mut MeleeAttack,
            &mut MeleeSwingType,
            &mut Transform,
        ),
        With<MeleeSwingMarker>,
    >,
) {
    for (entity, melee_attack, mut swing_type, mut weapon_hitbox_transform) in
        melee_query.iter_mut()
    {
        if let Ok(current_caster_transform) = caster_query.get(melee_attack.caster_entity) {
            match *swing_type {
                MeleeSwingType::Stab {
                    speed,
                    forward_duration,
                    return_duration,
                    mut elapsed_time,
                    max_distance,
                } => {
                    let direction = weapon_hitbox_transform.local_x().truncate();
                    let delta = direction * speed * time.delta_secs();

                    // Don't reset to caster position, just add the delta
                    weapon_hitbox_transform.translation += delta.extend(0.0);

                    warn!(
                        "New hitbox transform: {:?}",
                        weapon_hitbox_transform.translation
                    );
                }
                MeleeSwingType::Slash {
                    angular_speed,
                    radius,
                    mut current_angle,
                    duration,
                    elapsed_time,
                } => {}
                MeleeSwingType::Circle {
                    expansion_speed,
                    mut current_radius,
                    duration,
                    elapsed_time,
                } => {}
            }
        } else {
            // If the caster no longer exists, despawn the attack
            commands.entity(entity).despawn();
        }
    }
}
