use avian2d::prelude::*;
use bevy::prelude::*;

use super::equipment::prelude::{DirectionalPoses, EquipmentPose};
use crate::prelude::*;

/// Rotation that turns the upward-facing weapon sprites to point along the swing direction.
const MELEE_WEAPON_ROTATION: f32 = std::f32::consts::FRAC_PI_2;

/// Farthest target distance at which an AI holder swings a melee weapon.
const MELEE_WEAPON_REACH: f32 = 68.0;

/// Seconds a sword's stab takes.
const SWORD_SWING_SECONDS: f32 = 0.2;
/// Seconds the heavy slash both axes share takes.
const AXE_SWING_SECONDS: f32 = 0.3;
/// Seconds every melee weapon waits after a swing starts before it can swing again.
const MELEE_WEAPON_COOLDOWN: f32 = 0.4;

static MELEE_EQUIPMENT_POSES: DirectionalPoses = DirectionalPoses {
    up: EquipmentPose::above(0.0, -8.0, 30.0),
    down: EquipmentPose::behind(0.0, 8.0, -30.0),
    left: EquipmentPose::behind(-8.0, -15.0, 90.0),
    right: EquipmentPose::above(8.0, -15.0, -90.0),
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        process_melee_attacks.in_set(SimulationSystems::Derive),
    );
}

pub fn sword(sprites: &SpriteAssets) -> impl Scene {
    bsn! {
        @melee_weapon(sprites.sword.clone(), 120, SWORD_SWING_SECONDS)
        Name::new("Sword")
        ~{MeleeWeapon {
            damage: Damage::Range { min: 1.0, max: 6.0 },
            hitbox: Collider::rectangle(10.0, 40.0),
            swing_kind: MeleeSwingKind::STAB,
            hold_distance: 15.0,
        }}
        Knockback(2500.0)
    }
}

pub fn axe(sprites: &SpriteAssets) -> impl Scene {
    bsn! {
        @melee_weapon(sprites.axe.clone(), 220, AXE_SWING_SECONDS)
        Name::new("Axe")
        ~{axe_swing()}
        Knockback(5000.0)
    }
}

pub fn freeze_axe(sprites: &SpriteAssets) -> impl Scene {
    bsn! {
        @melee_weapon(sprites.axe.clone(), 220, AXE_SWING_SECONDS)
        Name::new("Freeze Axe")
        ~{axe_swing()}
        Knockback(500.0)
        Effects [
            Frozen
            Lifespan::new(2.0)
            --
            @stat_modifier(Stat::Speed, -1.0)
            Lifespan::new(2.0)
        ]
    }
}

/// The heavy slash both axes share.
fn axe_swing() -> MeleeWeapon {
    MeleeWeapon {
        damage: Damage::Range {
            min: 2.0,
            max: 12.0,
        },
        hitbox: Collider::rectangle(10.0, 40.0),
        swing_kind: MeleeSwingKind::SLASH,
        hold_distance: 30.0,
    }
}

/// What every melee weapon shares: a mainhand item worth `value`, drawn with `image`, that
/// swings for `swing_seconds` when used.
fn melee_weapon(image: Handle<Image>, value: u32, swing_seconds: f32) -> impl Scene {
    bsn! {
        ~Equippable::new(EquipmentSlot::Mainhand, &MELEE_EQUIPMENT_POSES)
        ~Cooldown::new(MELEE_WEAPON_COOLDOWN)
        ~{ActionReach(MELEE_WEAPON_REACH)}
        ~{ActionUseStyle::Timed { seconds: swing_seconds }}
        Attack
        ~Item::new(value, ItemKind::Melee)
        Sprite { image }
        on(on_melee_took_effect)
        on(on_melee_finished)
    }
}

/// A weapon that swings a hitbox along its holder's aim when used.
#[derive(Component, Clone, Debug)]
struct MeleeWeapon {
    damage: Damage,
    hitbox: Collider,
    swing_kind: MeleeSwingKind,
    hold_distance: f32,
}

#[derive(Clone, Copy, Debug)]
enum MeleeSwingKind {
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
    const STAB: Self = Self::Stab { reach: 30.0 };
    const SLASH: Self = Self::Slash {
        arc_distance: 180f32.to_radians(),
    };
}

/// Present on a melee weapon while it swings, for as long as its [`ActionUseStyle::Timed`] use.
#[derive(Component, Clone, Copy, Debug)]
struct ActiveMeleeAttack {
    /// Comes from the direction the entity holding the weapon is aiming.
    initial_angle: f32,
}

/// Spawns the swing's hitbox on the weapon for the duration of the use. Its damage is scaled by the
/// holder's stats at the time of the swing.
fn on_melee_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    weapon_query: Query<&MeleeWeapon>,
    holder_query: Query<(&Vision, &CombatSide, &StatMultipliers)>,
) -> Result {
    let melee_weapon = weapon_query.get(took_effect.source)?;
    let Ok((vision, side, stat_multipliers)) = holder_query.get(took_effect.holder) else {
        return Ok(());
    };

    let angle = vision.aim_direction.to_angle();
    let damage = melee_weapon.damage.scaled(stat_multipliers.damage);
    let weapon = took_effect.source;
    commands.entity(weapon).insert(ActiveMeleeAttack {
        initial_angle: angle,
    });
    commands.spawn((
        HitBox::new(took_effect.holder, damage, Vec2::from_angle(angle)).with_source_item(weapon),
        melee_weapon.hitbox.clone(),
        side.hitbox_layers(),
        Transform::default(),
        ChildOf(weapon),
        PartOfActionUse { source: weapon },
    ));
    Ok(())
}

/// Stops the swing.
fn on_melee_finished(finished: On<ActionFinished>, mut commands: Commands) {
    commands
        .entity(finished.source)
        .try_remove::<ActiveMeleeAttack>();
}

/// Moves each swinging melee weapon along its swing path, following the timed use's progress.
// TODO: Consider using bevy curve functions to reduce math needed here
fn process_melee_attacks(
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
