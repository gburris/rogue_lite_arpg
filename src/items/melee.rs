use avian2d::prelude::*;
use bevy::prelude::*;

use super::equipment::{DirectionalPoses, EquipmentPose};
use crate::interval::Interval;
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
            damage: Damage::Range {
                amount: Interval::new(1.0, 6.0),
            },
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
            amount: Interval::new(2.0, 12.0),
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
        ~{Reach(MELEE_WEAPON_REACH)}
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
    aim: Rot2,
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

    let aim = Dir2::new(vision.aim_direction).map_or(Rot2::IDENTITY, Dir2::rotation_from_x);
    let damage = melee_weapon.damage.scaled(stat_multipliers.damage);
    let weapon = took_effect.source;
    commands.entity(weapon).insert(ActiveMeleeAttack { aim });
    commands.spawn((
        HitBox::new(damage, aim * Vec2::X),
        CreditedAttackOf {
            attacker: took_effect.holder,
        },
        HitBoxOf { item: weapon },
        melee_weapon.hitbox.clone(),
        side.hitbox_layers(),
        Transform::default(),
        ChildOf(weapon),
        ActionUsePartOf {
            action_use: took_effect.action_use,
        },
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
fn process_melee_attacks(
    // Melee weapons are never projectiles or shields; the filter lets this run beside the systems
    // that move those.
    mut attack_query: Query<
        (
            &MeleeWeapon,
            &mut Transform,
            &ActiveMeleeAttack,
            &ActionSourceInUse,
        ),
        (Without<Projectile>, Without<Shield>),
    >,
    action_uses: Query<&TimedActionProgress>,
) {
    for (melee_weapon, mut transform, active_attack, source_in_use) in &mut attack_query {
        let Ok(timed_progress) = action_uses.get(source_in_use.get()) else {
            continue;
        };
        let attack_progress = timed_progress.fraction();
        let hold_distance = melee_weapon.hold_distance;

        let (sweep, distance) = match melee_weapon.swing_kind {
            MeleeSwingKind::Stab { reach } => (
                Rot2::IDENTITY,
                hold_distance.lerp(hold_distance + reach, attack_progress),
            ),
            MeleeSwingKind::Slash { arc_distance } => {
                let half_arc = arc_distance / 2.0;
                let sweep_angle = (-half_arc).lerp(half_arc, attack_progress);
                (Rot2::radians(sweep_angle), hold_distance)
            }
        };

        let swing_rotation = active_attack.aim * sweep;
        transform.translation = (swing_rotation * Vec2::X * distance).extend(0.0);
        transform.rotation =
            Quat::from_rotation_z(swing_rotation.as_radians() - MELEE_WEAPON_ROTATION);
    }
}
