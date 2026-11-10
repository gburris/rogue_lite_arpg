use avian2d::prelude::*;
use bevy::prelude::*;

use super::attack::{on_start_melee, on_stop_melee};
use super::swing::MeleeSwingKind;
use super::{MELEE_WEAPON_REACH, MeleeWeapon};
use crate::prelude::*;

pub fn sword(sprites: &SpriteAssets) -> impl Scene {
    bsn! {
        @melee_weapon(sprites.sword.clone(), 120)
        Name::new("Sword")
        ~{MeleeWeapon {
            damage: Damage::Range { min: 1.0, max: 6.0 },
            hitbox: Collider::rectangle(10.0, 40.0),
            swing_kind: MeleeSwingKind::STAB,
            attack_time: 0.2,
            hold_distance: 15.0,
        }}
        Knockback(10.0)
    }
}

pub fn axe(sprites: &SpriteAssets) -> impl Scene {
    bsn! {
        @melee_weapon(sprites.axe.clone(), 220)
        Name::new("Axe")
        ~{axe_swing()}
        Knockback(20.0)
    }
}

pub fn freeze_axe(sprites: &SpriteAssets) -> impl Scene {
    bsn! {
        @melee_weapon(sprites.axe.clone(), 220)
        Name::new("Freeze Axe")
        ~{axe_swing()}
        Knockback(2.0)
        Effects [
            Frozen
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
        attack_time: 0.3,
        hold_distance: 30.0,
    }
}

/// What every melee weapon shares: a mainhand item worth `value`, drawn with `image`, that
/// swings when used.
fn melee_weapon(image: Handle<Image>, value: u32) -> impl Scene {
    bsn! {
        ~Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_POSES)
            .with_reach(MELEE_WEAPON_REACH)
        ~Item::new(value, ItemKind::Melee)
        Sprite { image }
        on(on_stop_melee)
        on(on_start_melee)
    }
}

static MELEE_EQUIPMENT_POSES: DirectionalPoses = DirectionalPoses {
    up: EquipmentPose::above(0.0, -8.0, 30.0),
    down: EquipmentPose::behind(0.0, 8.0, -30.0),
    left: EquipmentPose::behind(-8.0, -15.0, 90.0),
    right: EquipmentPose::above(8.0, -15.0, -90.0),
};
