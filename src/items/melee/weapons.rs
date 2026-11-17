use avian2d::prelude::*;
use bevy::prelude::*;

use super::attack::{on_melee_finished, on_melee_took_effect};
use super::swing::MeleeSwingKind;
use super::{MELEE_WEAPON_REACH, MeleeWeapon};
use crate::items::equipment::prelude::{DirectionalPoses, EquipmentPose};
use crate::prelude::*;

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

/// Seconds a sword's stab takes.
const SWORD_SWING_SECONDS: f32 = 0.2;
/// Seconds the heavy slash both axes share takes.
const AXE_SWING_SECONDS: f32 = 0.3;
/// Seconds every melee weapon waits after a swing starts before it can swing again.
const MELEE_WEAPON_COOLDOWN: f32 = 0.4;

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
        ~ActionCooldown::new(MELEE_WEAPON_COOLDOWN)
        ~{ActionReach(MELEE_WEAPON_REACH)}
        ~{ActionUseStyle::Timed { seconds: swing_seconds }}
        Attack
        ~Item::new(value, ItemKind::Melee)
        Sprite { image }
        on(on_melee_took_effect)
        on(on_melee_finished)
    }
}

static MELEE_EQUIPMENT_POSES: DirectionalPoses = DirectionalPoses {
    up: EquipmentPose::above(0.0, -8.0, 30.0),
    down: EquipmentPose::behind(0.0, 8.0, -30.0),
    left: EquipmentPose::behind(-8.0, -15.0, 90.0),
    right: EquipmentPose::above(8.0, -15.0, -90.0),
};
