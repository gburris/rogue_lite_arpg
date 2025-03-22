use avian2d::prelude::Collider;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    combat::{
        mana::ManaCost,
        melee::{MeleeSwingType, MeleeWeapon},
        projectile::{
            components::{Projectile, ProjectileBundle},
            ProjectileWeapon,
        },
        status_effects::{
            components::{BurningStatus, EffectsList, StatusType},
            events::ApplyStatus,
        },
    },
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::{
        equipment::{on_weapon_fired, on_weapon_melee, Equippable},
        Item,
    },
};

use super::ItemType;

pub fn spawn_sword(commands: &mut Commands, sprites: &SpriteAssets) -> Entity {
    commands
        .spawn((
            MeleeWeapon {
                damage: (1.0, 6.0),
                effects_list: EffectsList { effects: vec![] },
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
                effects_list: EffectsList {
                    effects: vec![ApplyStatus {
                        status: StatusType::Frozen,
                        duration: 2.0,
                    }],
                },
                hitbox: Collider::rectangle(10.0, 40.0),
                attack_type: MeleeSwingType::SLASH,
                attack_time: 0.3,
                hold_distance: 30.0,
            },
            Name::new("Axe"),
            Equippable::default(),
            Item::new(220, ItemType::Melee),
            Sprite::from_image(sprites.axe.clone()),
        ))
        .observe(on_weapon_melee)
        .id()
}

pub fn spawn_fire_staff(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    texture_layouts: &SpriteSheetLayouts,
) -> Entity {
    let fireball = ProjectileBundle {
        projectile: Projectile {
            damage: (5.0, 10.0),
        },
        effects_list: EffectsList {
            effects: vec![ApplyStatus {
                status: StatusType::Burning(BurningStatus::default()),
                duration: 2.0,
            }],
        },
        sprite: Sprite::from_atlas_image(
            sprites.fire_ball.clone(),
            TextureAtlas {
                layout: texture_layouts.fireball_layout.clone(),
                index: 0,
            },
        ),
    };

    commands
        .spawn((
            ProjectileWeapon {
                projectile: fireball,
                projectile_speed: 700.0,
                spread: 0.0,
            },
            Name::new("Staff of Flames"),
            Item::new(1340, ItemType::Staff),
            Equippable::default(),
            ManaCost(6.0),
            Sprite::from_image(sprites.fire_staff.clone()),
        ))
        .observe(on_weapon_fired)
        .id()
}

pub fn spawn_ice_staff(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    texture_layouts: &SpriteSheetLayouts,
) -> Entity {
    let icicle_projectile = ProjectileBundle {
        projectile: Projectile {
            damage: (12.0, 25.0),
        }, // big damage
        effects_list: EffectsList {
            effects: vec![ApplyStatus {
                status: StatusType::Frozen,
                duration: 2.0,
            }],
        },
        sprite: Sprite::from_atlas_image(
            sprites.ice_bolt.clone(),
            TextureAtlas {
                layout: texture_layouts.ice_bolt_layout.clone(),
                index: 0,
            },
        ),
    };

    commands
        .spawn((
            ProjectileWeapon {
                projectile: icicle_projectile,
                projectile_speed: 500.0,
                spread: 0.0,
            },
            Name::new("Staff of Ice"),
            Item::new(2050, ItemType::Staff),
            ManaCost(20.0), // big mana cost
            Equippable {
                use_rate: Timer::from_seconds(0.7, TimerMode::Once),
                ..default()
            },
            Sprite::from_image(sprites.ice_staff.clone()),
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
