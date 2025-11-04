use bevy::prelude::*;

use super::{ActiveMeleeAttack, MELEE_WEAPON_ROTATION, MeleeWeapon};

#[derive(Debug, Clone)]
pub(super) enum MeleeSwingType {
    Stab {
        /// How far the weapon should move (stab) forward from its starting position
        reach: f32,
    },
    Slash {
        /// Distance we want slash to travel in radians
        arc_distance: f32,
    },
}

impl MeleeSwingType {
    pub const STAB: Self = MeleeSwingType::Stab { reach: 30.0 };
    pub const SLASH: Self = MeleeSwingType::Slash {
        arc_distance: 180f32.to_radians(),
    };
}

/// Determines path of melee weapon based on swing type
/// TODO: Consider using bevy curve functions to reduce math needed here
pub(super) fn process_melee_attacks(
    time: Res<Time>,
    mut attack_query: Query<(&MeleeWeapon, &mut Transform, &mut ActiveMeleeAttack)>,
) {
    for (melee_weapon, mut transform, mut active_attack) in attack_query.iter_mut() {
        active_attack.duration.tick(time.delta());
        let attack_progress = active_attack.duration.fraction();

        match melee_weapon.attack_type {
            MeleeSwingType::Stab { reach } => {
                // Total distance of stab * time of swing gets new position each tick
                let distance = reach * attack_progress;

                let forward = Vec2::from_angle(active_attack.initial_angle);
                let new_stab_position = forward * (melee_weapon.hold_distance + distance);

                transform.translation = new_stab_position.extend(0.0);
                transform.rotation =
                    Quat::from_rotation_z(active_attack.initial_angle - MELEE_WEAPON_ROTATION);
            }
            MeleeSwingType::Slash { arc_distance } => {
                let start_angle = active_attack.initial_angle - (arc_distance / 2.0);

                let current_angle = start_angle + (arc_distance * attack_progress);

                let new_axe_position = Vec2::from_angle(current_angle) * melee_weapon.hold_distance;

                transform.translation = new_axe_position.extend(0.0);
                transform.rotation = Quat::from_rotation_z(current_angle - MELEE_WEAPON_ROTATION);
            }
        }
    }
}
