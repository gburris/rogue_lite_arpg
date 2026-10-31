use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::ui_widgets::observe;

use super::attack::{on_start_melee, on_stop_melee};
use super::swing::MeleeSwingKind;
use super::{MELEE_WEAPON_REACH, MeleeWeapon};
use crate::prelude::*;

pub fn sword(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Sword"),
        MeleeWeapon {
            damage: Damage::Range { min: 1.0, max: 6.0 },
            hitbox: Collider::rectangle(10.0, 40.0),
            swing_kind: MeleeSwingKind::STAB,
            attack_time: 0.2,
            hold_distance: 15.0,
        },
        Knockback(10.0),
        Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_POSES)
            .with_reach(MELEE_WEAPON_REACH),
        Item::new(120, ItemKind::Melee),
        Sprite::from_image(sprites.sword.clone()),
        observe(on_stop_melee),
        observe(on_start_melee),
    )
}

pub fn axe(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Axe"),
        MeleeWeapon {
            damage: Damage::Range {
                min: 2.0,
                max: 12.0,
            },
            hitbox: Collider::rectangle(10.0, 40.0),
            swing_kind: MeleeSwingKind::SLASH,
            attack_time: 0.3,
            hold_distance: 30.0,
        },
        Knockback(20.0),
        Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_POSES)
            .with_reach(MELEE_WEAPON_REACH),
        Sprite::from_image(sprites.axe.clone()),
        Item::new(220, ItemKind::Melee),
        observe(on_stop_melee),
        observe(on_start_melee),
    )
}

pub fn freeze_axe(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Freeze Axe"),
        MeleeWeapon {
            damage: Damage::Range {
                min: 2.0,
                max: 12.0,
            },
            hitbox: Collider::rectangle(10.0, 40.0),
            swing_kind: MeleeSwingKind::SLASH,
            attack_time: 0.3,
            hold_distance: 30.0,
        },
        Knockback(2.0),
        Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_POSES)
            .with_reach(MELEE_WEAPON_REACH),
        Item::new(220, ItemKind::Melee),
        Sprite::from_image(sprites.axe.clone()),
        related!(Effects[(Frozen, Lifespan::new(2.0))]),
        observe(on_stop_melee),
        observe(on_start_melee),
    )
}

static MELEE_EQUIPMENT_POSES: DirectionalPoses = DirectionalPoses {
    up: EquipmentPose::above(0.0, -8.0, 30.0),
    down: EquipmentPose::behind(0.0, 8.0, -30.0),
    left: EquipmentPose::behind(-8.0, -15.0, 90.0),
    right: EquipmentPose::above(8.0, -15.0, -90.0),
};
