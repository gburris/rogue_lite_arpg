use bevy::prelude::*;

use super::{ActiveMeleeAttack, MELEE_WEAPON_ROTATION, MeleeWeapon};
use crate::prelude::*;

/// How far a stab draws back toward its holder over a wind-up.
const STAB_DRAW_BACK: f32 = 10.0;
/// How far a slash draws back past its starting angle over a wind-up, in radians.
const SLASH_DRAW_BACK: f32 = 0.6;
/// The weapon's color while it winds up, so its target can see the blow coming.
const WIND_UP_TINT: Color = Color::srgb(1.0, 0.35, 0.35);

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

    // TODO: Consider using bevy curve functions to reduce math needed here
    /// Where the weapon is `progress` of the way through a swing aimed along `aim_angle`.
    fn swing_pose(self, hold_distance: f32, aim_angle: f32, progress: f32) -> WeaponPose {
        match self {
            Self::Stab { reach } => WeaponPose {
                angle: aim_angle,
                distance: hold_distance + reach * progress,
            },
            Self::Slash { arc_distance } => WeaponPose {
                angle: aim_angle - arc_distance / 2.0 + arc_distance * progress,
                distance: hold_distance,
            },
        }
    }

    /// Where the weapon is `progress` of the way through a wind-up: drawn back from where the
    /// swing starts.
    fn wind_up_pose(self, hold_distance: f32, aim_angle: f32, progress: f32) -> WeaponPose {
        match self {
            Self::Stab { .. } => WeaponPose {
                angle: aim_angle,
                distance: hold_distance - STAB_DRAW_BACK * progress,
            },
            Self::Slash { arc_distance } => WeaponPose {
                angle: aim_angle - arc_distance / 2.0 - SLASH_DRAW_BACK * progress,
                distance: hold_distance,
            },
        }
    }
}

/// A weapon's place around its holder: the direction it points and how far out it is held.
#[derive(Clone, Copy, Debug)]
struct WeaponPose {
    angle: f32,
    distance: f32,
}

impl WeaponPose {
    fn apply_to(self, transform: &mut Transform) {
        transform.translation = (Vec2::from_angle(self.angle) * self.distance).extend(0.0);
        transform.rotation = Quat::from_rotation_z(self.angle - MELEE_WEAPON_ROTATION);
    }
}

/// Moves each attacking weapon through its wind-up, tinted, and then through its swing, giving it
/// its hitbox and putting out its wind-up glow once the wind-up is over.
pub(super) fn process_melee_attacks(
    mut commands: Commands,
    time: Res<Time>,
    // Melee weapons are never projectiles; the filter lets this run beside projectile systems.
    mut attack_query: Query<
        (
            Entity,
            &MeleeWeapon,
            &mut Transform,
            &mut Sprite,
            &mut ActiveMeleeAttack,
        ),
        Without<Projectile>,
    >,
) {
    for (weapon, melee_weapon, mut transform, mut sprite, mut active_attack) in &mut attack_query {
        let aim_angle = active_attack.initial_angle;

        if let Some(wind_up) = active_attack.wind_up.as_mut()
            && !wind_up.is_finished()
        {
            wind_up.tick(time.delta());
            if !wind_up.is_finished() {
                sprite.color = WIND_UP_TINT;
                melee_weapon
                    .swing_kind
                    .wind_up_pose(melee_weapon.hold_distance, aim_angle, wind_up.fraction())
                    .apply_to(&mut transform);
                continue;
            }
            sprite.color = Color::WHITE;
            commands
                .entity(weapon)
                .insert(melee_weapon.hitbox.clone())
                .despawn_children();
        }

        active_attack.duration.tick(time.delta());
        melee_weapon
            .swing_kind
            .swing_pose(
                melee_weapon.hold_distance,
                aim_angle,
                active_attack.duration.fraction(),
            )
            .apply_to(&mut transform);
    }
}
