use std::sync::LazyLock;

use avian2d::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::ui_widgets::observe;

use super::attack::{on_start_melee, on_stop_melee};
use super::swing::MeleeSwingType;
use super::{MELEE_WEAPON_REACH, MeleeWeapon};
use crate::equipment_transforms;
use crate::prelude::*;

pub fn sword(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Sword"),
        MeleeWeapon {
            damage: (1.0, 6.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::STAB,
            attack_time: 0.2,
            hold_distance: 15.0,
        },
        Knockback(10.0),
        Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_TRANSFORMS)
            .with_reach(MELEE_WEAPON_REACH),
        Item::new(120, ItemType::Melee),
        Sprite::from_image(sprites.sword.clone()),
        observe(on_stop_melee),
        observe(on_start_melee),
    )
}

pub fn axe(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Axe"),
        MeleeWeapon {
            damage: (2.0, 12.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::SLASH,
            attack_time: 0.3,
            hold_distance: 30.0,
        },
        Knockback(20.0),
        Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_TRANSFORMS)
            .with_reach(MELEE_WEAPON_REACH),
        Sprite::from_image(sprites.axe.clone()),
        Item::new(220, ItemType::Melee),
        observe(on_stop_melee),
        observe(on_start_melee),
    )
}

pub fn freeze_axe(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Freeze Axe"),
        MeleeWeapon {
            damage: (2.0, 12.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::SLASH,
            attack_time: 0.3,
            hold_distance: 30.0,
        },
        Knockback(2.0),
        Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_TRANSFORMS)
            .with_reach(MELEE_WEAPON_REACH),
        Item::new(220, ItemType::Melee),
        Sprite::from_image(sprites.axe.clone()),
        related!(Effects[(Frozen, Lifespan::new(2.0))]),
        observe(on_stop_melee),
        observe(on_start_melee),
    )
}

use ZLayer::BehindSprite;
static MELEE_EQUIPMENT_TRANSFORMS: LazyLock<HashMap<FacingDirection, Transform>> =
    equipment_transforms!([
        (Up, ((0.0, -8.0), 30.0)),
        (Down, ((0.0, 8.0), -30.0, BehindSprite)),
        (Left, ((-8.0, -15.0), 90.0, BehindSprite)),
        (Right, ((8.0, -15.0), -90.0)),
    ]);
