use avian2d::prelude::Collider;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

use super::{
    equipment::{
        equipment_transform::EquipmentTransform,
        use_equipped::{on_weapon_fired, on_weapon_melee},
        Equippable,
    },
    grounded::handle_item_to_ground::handle_item_ground_transition,
};
use crate::{
    animation::FacingDirection,
    combat::{
        attributes::mana::ManaCost,
        melee::components::{MeleeSwingType, MeleeWeapon},
        projectile::components::{Projectile, ProjectileBundle},
        status_effects::{
            components::{BurningStatus, EffectsList, StatusType},
            events::ApplyStatus,
        },
        weapon::weapon::ProjectileWeapon,
    },
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::Item,
};

pub fn spawn_sword(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            MeleeWeapon {
                damage: 6.0,
                effects_list: EffectsList { effects: vec![] },
                hitbox: Collider::rectangle(10.0, 40.0),
                attack_type: MeleeSwingType::stab(),
                attack_duration: Timer::from_seconds(0.4, TimerMode::Once),
            },
            Name::new("Sword"),
            Equippable::default(),
            Item::new(3),
            Visibility::Visible,
            Sprite::from_image(sprites.sword.clone()),
            EquipmentTransform::get(FacingDirection::Down).mainhand,
        ))
        .observe(on_weapon_melee)
        .observe(handle_item_ground_transition)
        .id()
}

pub fn spawn_axe(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            MeleeWeapon {
                damage: 60.0,
                effects_list: EffectsList {
                    effects: vec![ApplyStatus {
                        status: StatusType::Frozen,
                        duration: 2.0,
                    }],
                },
                hitbox: Collider::rectangle(10.0, 40.0),
                attack_type: MeleeSwingType::slash(),
                attack_duration: Timer::from_seconds(0.4, TimerMode::Once),
            },
            Name::new("Axe"),
            Equippable::default(),
            Item::new(4),
            Visibility::Visible,
            Sprite::from_image(sprites.axe.clone()),
            EquipmentTransform::get(FacingDirection::Down).mainhand,
        ))
        .observe(on_weapon_melee)
        .observe(handle_item_ground_transition)
        .id()
}

pub fn spawn_shovel(commands: &mut Commands, sprites: &Res<SpriteAssets>) -> Entity {
    commands
        .spawn((
            MeleeWeapon {
                damage: 6.0,
                effects_list: EffectsList { effects: vec![] },
                hitbox: Collider::rectangle(10.0, 40.0),
                attack_type: MeleeSwingType::stab(),
                attack_duration: Timer::from_seconds(0.4, TimerMode::Once),
            },
            Name::new("Shovel"),
            Equippable::default(),
            Item::new(5),
            Visibility::Hidden,
            Sprite::from_image(sprites.shovel_equipment_sprite.clone()),
            EquipmentTransform::get(FacingDirection::Down).mainhand,
        ))
        .observe(handle_item_ground_transition)
        .observe(on_weapon_melee)
        .id()
}

pub fn spawn_fire_staff(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    texture_layouts: &Res<SpriteSheetLayouts>,
) -> Entity {
    let fireball = ProjectileBundle {
        projectile: Projectile { damage: 6.0 },
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

    let weapon_transform: Transform = EquipmentTransform::get(FacingDirection::Down).mainhand;

    commands
        .spawn((
            ProjectileWeapon {
                projectile: fireball,
                projectile_speed: 700.0,
                spread: 0.0,
            },
            Name::new("Staff of Flames"),
            Item::new(6),
            Equippable::default(),
            ManaCost(6.0),
            Visibility::Hidden,
            Sprite::from_image(sprites.fire_staff.clone()),
            weapon_transform,
        ))
        .observe(on_weapon_fired)
        .observe(handle_item_ground_transition)
        .id()
}

pub fn spawn_ice_staff(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    texture_layouts: &Res<SpriteSheetLayouts>,
) -> Entity {
    let icicle_projectile = ProjectileBundle {
        projectile: Projectile { damage: 25.0 }, // big damage
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

    let weapon_transform: Transform = EquipmentTransform::get(FacingDirection::Down).mainhand;

    commands
        .spawn((
            ProjectileWeapon {
                projectile: icicle_projectile,
                projectile_speed: 500.0,
                spread: 0.0,
            },
            Name::new("Staff of Ice"),
            Item::new(7),
            ManaCost(20.0), // big mana cost
            Equippable {
                use_rate: Timer::from_seconds(0.7, TimerMode::Once),
                ..default()
            },
            Visibility::Hidden,
            Sprite::from_image(sprites.ice_staff.clone()),
            weapon_transform,
        ))
        .observe(on_weapon_fired)
        .observe(handle_item_ground_transition)
        .id()
}

pub fn spawn_random_mainhand_weapon(
    commands: &mut Commands,
    sprites: &Res<SpriteAssets>,
    texture_layouts: &Res<SpriteSheetLayouts>,
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
