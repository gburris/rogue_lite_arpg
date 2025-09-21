use std::f32::consts::FRAC_PI_8;

use avian2d::prelude::Collider;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    combat::{
        mana::ManaCost,
        melee::{MeleeSwingType, MeleeWeapon},
        projectile::{BulletSprite, ProjectileBuilder, Projectiles},
        status_effects::{Burning, Effects, Frozen},
    },
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::{
        equipment::{on_weapon_fired, on_weapon_melee, Equippable},
        Item,
    },
    utility::Lifespan,
};

use super::ItemType;

pub fn spawn_sword(commands: &mut Commands, sprites: &SpriteAssets) -> Entity {
    commands
        .spawn((
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
        ))
        .observe(on_weapon_melee)
        .id()
}

pub fn spawn_axe(commands: &mut Commands, sprites: &SpriteAssets) -> Entity {
    commands
        .spawn((
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
        ))
        .observe(on_weapon_melee)
        .id()
}

pub fn spawn_fire_staff(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    texture_layouts: &SpriteSheetLayouts,
) -> Entity {
    let fireball_builder = ProjectileBuilder::new(BulletSprite::Fireball, sprites, texture_layouts);
    let burning = (Burning::default(), Lifespan::new(1.2));

    commands
        .spawn((
            Name::new("Staff of Flames"),
            Item::new(1340, ItemType::Staff),
            Equippable::default(),
            ManaCost(6.0),
            Sprite::from_image(sprites.fire_staff.clone()),
            related!(
                Projectiles [
                    (fireball_builder.clone().with_angle_offset(-FRAC_PI_8).build(), related!(Effects[burning.clone()])),
                    (fireball_builder.clone().build(), related!(Effects[burning.clone()])),
                    (fireball_builder.clone().with_angle_offset(FRAC_PI_8).build(), related!(Effects[burning.clone()])),
                ]
            ),
        ))
        .observe(on_weapon_fired)
        .id()
}

pub fn spawn_ice_staff(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    texture_layouts: &SpriteSheetLayouts,
) -> Entity {
    commands
        .spawn((
            Name::new("Staff of Ice"),
            Item::new(2050, ItemType::Staff),
            ManaCost(20.0), // big mana cost
            Equippable {
                use_rate: Timer::from_seconds(0.7, TimerMode::Once),
                ..default()
            },
            Sprite::from_image(sprites.ice_staff.clone()),
            Projectiles::spawn_one((
                ProjectileBuilder::new(BulletSprite::IceBolt, sprites, texture_layouts).build(),
                related!(Effects[(Frozen, Lifespan::new(2.0))]),
            )),
        ))
        .observe(on_weapon_fired)
        .id()
}

pub fn spawn_random_mainhand_weapon(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    texture_layouts: &SpriteSheetLayouts,
) -> Entity {
    let mut rng = thread_rng();
    let choice = rng.gen_range(0..4);

    match choice {
        0 => spawn_sword(commands, sprites),
        1 => spawn_axe(commands, sprites),
        2 => spawn_fire_staff(commands, sprites, texture_layouts),
        3 => spawn_ice_staff(commands, sprites, texture_layouts),
        _ => unreachable!(), // Should never happen
    }
}

//TODO: Everything in this class is private except this,
//And make this more of a factory pattern kinda vibe
pub fn spawn_mainhand_weapon(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    texture_layouts: &SpriteSheetLayouts,
    weapon_name: &str,
) -> Entity {
    match weapon_name {
        "sword" => spawn_sword(commands, sprites),
        "axe" => spawn_axe(commands, sprites),
        "fire_staff" => spawn_fire_staff(commands, sprites, texture_layouts),
        "ice_staff" => spawn_ice_staff(commands, sprites, texture_layouts),
        _ => unreachable!(), // Should never happen
    }
}
