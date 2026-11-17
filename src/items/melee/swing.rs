use bevy::prelude::*;

use super::attack::ActiveMeleeAttack;
use super::{MELEE_WEAPON_ROTATION, MeleeWeapon};
use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum MeleeSwingKind {
    Stab {
        /// How far the weapon should move (stab) forward from its starting position.
        reach: f32,
    },
    Slash {
        /// Distance we want slash to travel in radians.
        arc_distance: f32,
    },
}

impl MeleeSwingKind {
    pub const STAB: Self = Self::Stab { reach: 30.0 };
    pub const SLASH: Self = Self::Slash {
        arc_distance: 180f32.to_radians(),
    };
}

/// Determines path of melee weapon based on swing type, along the timed use's progress.
// TODO: Consider using bevy curve functions to reduce math needed here
pub(super) fn process_melee_attacks(
    // Melee weapons are never projectiles or shields; the filter lets this run beside the systems
    // that move those.
    mut attack_query: Query<
        (
            &MeleeWeapon,
            &mut Transform,
            &ActiveMeleeAttack,
            &TimedActionProgress,
        ),
        (Without<Projectile>, Without<Shield>),
    >,
) {
    for (melee_weapon, mut transform, active_attack, timed_progress) in &mut attack_query {
        let attack_progress = timed_progress.fraction();

        match melee_weapon.swing_kind {
            MeleeSwingKind::Stab { reach } => {
                // Total distance of stab * time of swing gets new position each tick
                let distance = reach * attack_progress;

                let forward = Vec2::from_angle(active_attack.initial_angle);
                let new_stab_position = forward * (melee_weapon.hold_distance + distance);

                transform.translation = new_stab_position.extend(0.0);
                transform.rotation =
                    Quat::from_rotation_z(active_attack.initial_angle - MELEE_WEAPON_ROTATION);
            }
            MeleeSwingKind::Slash { arc_distance } => {
                let start_angle = active_attack.initial_angle - (arc_distance / 2.0);

                let current_angle = start_angle + (arc_distance * attack_progress);

                let new_axe_position = Vec2::from_angle(current_angle) * melee_weapon.hold_distance;

                transform.translation = new_axe_position.extend(0.0);
                transform.rotation = Quat::from_rotation_z(current_angle - MELEE_WEAPON_ROTATION);
            }
        }
    }
}
