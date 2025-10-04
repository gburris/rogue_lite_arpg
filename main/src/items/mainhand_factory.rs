use std::f32::consts::FRAC_PI_8;

use avian2d::prelude::Collider;
use bevy::prelude::*;
use bevy_bundled_observers::observers;

use crate::{
    combat::{
        mana::ManaCost,
        melee::{MeleeSwingType, MeleeWeapon},
        projectile::{fireball, icebolt, Projectiles},
        status_effects::{Effects, Frozen},
    },
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::{
        equipment::{on_weapon_fired, on_weapon_melee, Equippable},
        Item,
    },
    utility::Lifespan,
};

use super::ItemType;

pub fn sword(sprites: &SpriteAssets) -> impl Bundle {
    (
        MeleeWeapon {
            damage: (1.0, 6.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::STAB,
            attack_time: 0.2,
            hold_distance: 15.0,
        },
        Name::new("Sword"),
        Equippable::default(),
        Item::new(120, ItemType::Melee),
        Sprite::from_image(sprites.sword.clone()),
        observers![on_weapon_melee],
    )
}

pub fn axe(sprites: &SpriteAssets) -> impl Bundle {
    (
        MeleeWeapon {
            damage: (2.0, 12.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::SLASH,
            attack_time: 0.3,
            hold_distance: 30.0,
        },
        Name::new("Axe"),
        Equippable::default(),
        Item::new(220, ItemType::Melee),
        Sprite::from_image(sprites.axe.clone()),
        related!(Effects[(Frozen, Lifespan::new(2.0))]),
        observers![on_weapon_melee],
    )
}

pub fn fire_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Flames"),
        Item::new(1340, ItemType::Staff),
        Equippable::default(),
        ManaCost(6.0),
        Sprite::from_image(sprites.fire_staff.clone()),
        related!(
            Projectiles [
                fireball(sprites, sprite_layouts, -FRAC_PI_8),
                fireball(sprites, sprite_layouts, 0.0),
                fireball(sprites, sprite_layouts, FRAC_PI_8)
            ]
        ),
        observers![on_weapon_fired],
    )
}

pub fn ice_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Ice"),
        Item::new(2050, ItemType::Staff),
        ManaCost(20.0), // big mana cost
        Equippable {
            use_rate: Timer::from_seconds(0.7, TimerMode::Once),
            ..default()
        },
        Sprite::from_image(sprites.ice_staff.clone()),
        Projectiles::spawn_one(icebolt(sprites, sprite_layouts, 0.0)),
        observers![on_weapon_fired],
    )
}
